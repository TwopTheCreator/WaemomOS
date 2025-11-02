use core::ptr::{read_volatile, write_volatile};
use lazy_static::lazy_static;
use spin::Mutex;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

fn pci_config_address(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    (1u32 << 31) | ((bus as u32) << 16) | ((slot as u32) << 11) | ((func as u32) << 8) | ((offset as u32) & 0xFC)
}

unsafe fn outl(port: u16, val: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nostack, preserves_flags));
}
unsafe fn inl(port: u16) -> u32 {
    let mut val: u32;
    core::arch::asm!("in eax, dx", in("dx") port, out("eax") val, options(nostack, preserves_flags));
    val
}

pub fn read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    unsafe {
        outl(PCI_CONFIG_ADDRESS, pci_config_address(bus, slot, func, offset));
        inl(PCI_CONFIG_DATA)
    }
}

pub fn write_u32(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
    unsafe {
        outl(PCI_CONFIG_ADDRESS, pci_config_address(bus, slot, func, offset));
        outl(PCI_CONFIG_DATA, val);
    }
}

pub struct PciDevice { pub bus: u8, pub slot: u8, pub func: u8, pub vendor: u16, pub device: u16, pub class: u8, pub subclass: u8 }

pub fn enumerate(mut f: impl FnMut(PciDevice)) {
    for bus in 0..=255u8 { for slot in 0..32u8 { for func in 0..8u8 {
        let id = read_u32(bus, slot, func, 0x00);
        if id == 0xFFFF_FFFF { continue; }
        let vendor = (id & 0xFFFF) as u16;
        let device = ((id >> 16) & 0xFFFF) as u16;
        let class_reg = read_u32(bus, slot, func, 0x08);
        let class = ((class_reg >> 24) & 0xFF) as u8;
        let subclass = ((class_reg >> 16) & 0xFF) as u8;
        f(PciDevice{bus,slot,func,vendor,device,class,subclass});
    }}}
}
