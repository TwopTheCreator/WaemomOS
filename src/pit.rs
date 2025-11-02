use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::instructions::port::Port;
use lazy_static::lazy_static;
use spin::Mutex;

static TICKS: AtomicU64 = AtomicU64::new(0);
static mut HZ: u32 = 100;

pub fn init(hz: u32) {
    unsafe { HZ = hz; }
    let divisor: u16 = (1193182u32 / hz) as u16;
    unsafe {
        let mut cmd = Port::<u8>::new(0x43);
        cmd.write(0x36); // channel 0, lobyte/hibyte, mode 3
        let mut ch0 = Port::<u8>::new(0x40);
        ch0.write((divisor & 0xFF) as u8);
        ch0.write((divisor >> 8) as u8);
    }
}

pub fn tick() { TICKS.fetch_add(1, Ordering::Relaxed); crate::scheduler::on_tick(); crate::net::netstack::poll(); }

pub fn uptime_secs() -> u64 { let t = TICKS.load(Ordering::Relaxed); let hz = unsafe { HZ as u64 }; t / hz }

pub fn format_uptime() -> heapless::String<32> {
    let s = uptime_secs();
    let h = s / 3600; let m = (s % 3600) / 60; let sec = s % 60;
    heapless::String::from(format!("{:02}:{:02}:{:02}", h, m, sec))
}
