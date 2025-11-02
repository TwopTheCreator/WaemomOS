use core::sync::atomic::{AtomicI32, Ordering};
use x86_64::instructions::port::Port;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! { static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new()); }

pub struct Mouse { x: i32, y: i32, byte_idx: u8, packet: [u8;3] }
impl Mouse {
    const fn new() -> Self { Self { x: 100, y: 100, byte_idx: 0, packet: [0;3] } }
}

pub fn init() {
    // Enable auxiliary device and interrupts on PS/2 controller
    unsafe {
        let mut cmd = Port::<u8>::new(0x64);
        let mut data = Port::<u8>::new(0x60);
        // Enable aux port
        cmd.write(0xA8);
        // Enable interrupts for mouse
        cmd.write(0x20); // read command byte
        let mut status: u8 = data.read();
        status |= 0x02; // enable IRQ12
        cmd.write(0x60); data.write(status);
        // Reset mouse to default
        send_mouse(0xF6);
        send_mouse(0xF4); // enable data reporting
    }
}

unsafe fn send_mouse(byte: u8) {
    let mut cmd = Port::<u8>::new(0x64);
    let mut data = Port::<u8>::new(0x60);
    // tell controller next byte is for mouse
    cmd.write(0xD4);
    data.write(byte);
    let _ack: u8 = data.read();
}

pub fn on_irq() {
    unsafe {
        let mut data = Port::<u8>::new(0x60);
        let b = data.read();
        let mut ms = MOUSE.lock();
        ms.packet[ms.byte_idx as usize] = b;
        ms.byte_idx = (ms.byte_idx + 1) % 3;
        if ms.byte_idx == 0 {
            // decode
            let dx = (ms.packet[1] as i8) as i32;
            let dy = (ms.packet[2] as i8) as i32;
            ms.x = (ms.x + dx).max(0);
            ms.y = (ms.y - dy).max(0);
        }
    }
}

pub fn draw_cursor() {
    use crate::graphics::{self, Color};
    let ms = MOUSE.lock();
    graphics::fill_rect(ms.x as usize, ms.y as usize, 6, 6, Color::WHITE);
}
