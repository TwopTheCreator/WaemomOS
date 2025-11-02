#![no_std]
#![no_main]

extern crate alloc;

mod vga_buffer;
mod serial;
mod graphics;
mod window;
mod fs;
mod block;
mod ata;
mod elf;
mod heap;
mod settings;
mod image;
mod ui { pub mod icons; }
mod appmgr;
mod loader;
mod net { pub mod wifi; pub mod ip; pub mod crypto; pub mod e1000; pub mod netstack; pub use super::net::*; }
mod pci;
mod interrupts;
mod pit;
mod keyboard;
mod mouse;
mod console;
mod gdt;
mod syscalls;
mod tty;
mod shell;
mod mm;
mod elfloader;
mod context;
mod task;
mod scheduler;
use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootInfo};
use x86_64::instructions::hlt;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Init text and serial output
    println!("waemom kernel booting...");
    serial_println!("waemom: serial log initialized.");

    // Init heap allocator
    heap::init();

    // CPU tables
    gdt::init();

    // Initialize interrupts (PIC, IDT), PIT, keyboard, mouse
    interrupts::init();
    pit::init(100); // 100 Hz
    mouse::init();

    // Init scheduler
    scheduler::init();
    // Start shell task
    scheduler::spawn_kernel("shell", crate::shell::shell_task);

    // Init memory management (frames/page tables)
    mm::init(boot_info);

    // Init graphics framebuffer (if available)
    if let Some(_) = graphics::init(boot_info) {
        graphics::clear_screen(graphics::Color::rgb(32, 34, 36));

        // Initialize in-memory filesystem and demo files
        fs::init();

        // Load settings from waemon.lock if present
        if let Ok(lock) = fs::read("/waemon.lock") {
            if let Ok(s) = core::str::from_utf8(&lock) { settings::load_from_lock(s); }
        }
        let cfg = settings::current();

        // Optional loading animation
        if settings::current().loading_animations { loader::loading_sequence(&["/README.txt","/etc/motd","/waemon.lock","/www/index.html","/www/app.js","/www/styles.css","/bin/hello"]) }

        // Init e1000 NIC if present
        let _ = crate::net::e1000::E1000::init();
        // Connect to default Wi‑Fi (demo)
        apps::network::connect_default();

        // Desktop chrome
        window::refresh_desktop();
        // Draw console overlay initially
        console::draw();

        // Sample apps content
        let files_list = fs::list("/").unwrap_or_default();
        let notes = apps::notes();
        let files = apps::files(&files_list);
        let about = apps::about();
        let settings_view = apps::settings::view(&cfg);
        let tasks_view = apps::taskmgr::view(&[
            apps::taskmgr::TaskInfo{ pid:1,name:"Notes".into(),state:"running"},
            apps::taskmgr::TaskInfo{ pid:2,name:"Files".into(),state:"running"},
            apps::taskmgr::TaskInfo{ pid:3,name:"Settings".into(),state:"running"},
        ]);

        // Animated windows with icons
        window::open_window_icon_animated(20, 30, 420, 200, "Notes", notes, 18, ui::icons::icon_notes());
        window::open_window_icon_animated(460, 60, 420, 220, "Files", &files, 18, ui::icons::icon_files());

        // Linux ELF inspection (controlled by settings)
        if cfg.elf_enabled {
            if let Some(bytes) = fs::read("/bin/hello").ok() {
                let slice = &bytes[..bytes.len().min(cfg.elf_max_bytes)];
                let info = elf::inspect_elf64(slice);
                window::open_window_icon_animated(220, 180, 460, 220, "ELF64 Inspector", &info, 18, ui::icons::icon_files());
            } else {
                window::open_window_icon_animated(220, 180, 460, 160, "Linux Support", "ELF64 parser ready.", 18, ui::icons::icon_files());
            }
        }

        // Settings
        window::open_window_icon_animated(80, 320, 420, 180, "System Settings", &settings_view, 14, ui::icons::icon_settings());

        // Task Manager
        window::open_window_icon_animated(520, 320, 420, 200, "Task Manager", &tasks_view, 14, ui::icons::icon_task());

        // Network
        let net_view = apps::network::view();
        window::open_window_icon_animated(980, 320, 420, 160, "Network", &net_view, 14, ui::icons::icon_task());

        // Try mount FAT root (if available) and list entries to console
        let mut ata = crate::block::AtaDevice::new();
        let entries = crate::fs::fat::list_root(&mut ata);
        if !entries.is_empty() {
            crate::console::println("FAT root:");
            for e in entries { crate::console::println(e.as_str()); }
        }

        // Broom browser demo (also register in app manager with states)
        apps::broom::launch_demo();

        // Demonstrate app states
        use appmgr::{spawn, set_state, WindowState};
        let id_notes = spawn("Notes", 20, 30, 420, 200, ui::icons::icon_notes(), notes.to_string());
        let id_files = spawn("Files", 460, 60, 420, 220, ui::icons::icon_files(), files.clone());
        set_state(id_notes, WindowState::Minimized);
        set_state(id_files, WindowState::Maximized);

        // Web viewers if enabled
        if cfg.web_enabled {
            window::open_window_icon_animated(40, 520, 540, 220, "HTML", &apps::web::view_html(), 12, ui::icons::icon_files());
            window::open_window_icon_animated(600, 520, 520, 220, "JS", &apps::web::view_js(), 12, ui::icons::icon_files());
            window::open_window_icon_animated(1140, 520, 480, 220, "CSS", &apps::web::view_css(), 12, ui::icons::icon_files());
        }

    } else {
        println!("No framebuffer found; staying in text mode.");
    }

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("PANIC: {}", info);
    println!("PANIC: {}", info);
    hlt_loop();
}

#[inline(always)]
fn hlt_loop() -> ! {
    loop { hlt(); }
}
