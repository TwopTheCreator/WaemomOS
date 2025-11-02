use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::{interrupts, port::Port};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    // PIC2
    Mouse = PIC_1_OFFSET + 12,
    Syscall = 0x80,
}
impl InterruptIndex { fn as_u8(self) -> u8 { self as u8 } fn as_usize(self) -> usize { self.as_u8() as usize } }

lazy_static! { static ref IDT: InterruptDescriptorTable = {
    let mut idt = InterruptDescriptorTable::new();
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler);
    idt[InterruptIndex::Syscall.as_usize()].set_handler_fn(syscall_interrupt_handler);
    idt
}; }

pub fn init() {
    unsafe { PICS.lock().initialize() };
    IDT.load();
    unsafe { interrupts::enable(); }
}

pub fn notify_end_of_interrupt(idx: InterruptIndex) {
    unsafe { PICS.lock().notify_end_of_interrupt(idx.as_u8()); }
}

lazy_static! { pub static ref PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }); }

extern "x86-interrupt" fn timer_interrupt_handler(_stack: InterruptStackFrame) {
    crate::pit::tick();
    notify_end_of_interrupt(InterruptIndex::Timer);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack: InterruptStackFrame) {
    let mut port: Port<u8> = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::keyboard::on_scancode(scancode);
    notify_end_of_interrupt(InterruptIndex::Keyboard);
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack: InterruptStackFrame) {
    crate::mouse::on_irq();
    notify_end_of_interrupt(InterruptIndex::Mouse);
}

extern "x86-interrupt" fn syscall_interrupt_handler(_stack: InterruptStackFrame) {
    crate::syscalls::handle();
}
