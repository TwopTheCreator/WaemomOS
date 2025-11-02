use heapless::String;

pub extern "C" fn shell_task() -> ! {
    let mut buf = String::<256>::new();
    loop {
        if crate::tty::read_line(&mut buf) {
            handle_cmd(&buf);
            buf.clear();
        }
        crate::window::sleep(50_000);
    }
}

fn handle_cmd(cmd: &str) {
    match cmd.trim() {
        "hello" => crate::console::println("Hello!"),
        "clear" => crate::console::clear(),
        "uptime" => { let t = crate::pit::format_uptime(); crate::console::println(t.as_str()); },
        s if s.starts_with("sleep ") => {
            if let Ok(t) = s[6..].trim().parse::<u64>() { crate::syscalls::sleep_ticks(t); crate::console::println("(sleep)"); }
        }
        s if s.starts_with("spawn ") => {
            let name = &s[6..]; let _ = crate::syscalls::spawn(name); crate::console::println("(spawn)");
        }
        s if s.starts_with("run ") => {
            let path = &s[4..];
            let _ = crate::syscalls::spawn_user_elf(path);
            crate::console::println("(run)");
        }
        s if s.is_empty() => {}
        _ => crate::console::println("Unknown"),
    }
}
