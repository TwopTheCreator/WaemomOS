#![no_std]

pub use crate::console::{println as console_println, clear as console_clear};

pub mod console;
pub mod graphics;
pub mod serial;
pub mod vga_buffer;
pub mod window;
pub mod fs;
pub mod elf;
pub mod heap;
pub mod apps;
pub mod settings;
pub mod image;
pub mod ui { pub mod icons; }
pub mod appmgr;
pub mod loader;
pub mod net { pub mod wifi; pub mod ip; pub mod crypto; pub mod e1000; pub mod netstack; pub use super::net::*; }
pub mod pci;
pub mod interrupts;
pub mod pit;
pub mod keyboard;
pub mod syscalls;
pub mod tty;
pub mod shell;
pub mod gdt;
pub mod context;
pub mod task;
pub mod scheduler;