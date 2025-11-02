use crate::pci;
use core::ptr::{read_volatile, write_volatile};
use lazy_static::lazy_static;
use spin::Mutex;

pub struct E1000 {
    pub regs: *mut u32,
    pub mac: [u8;6],
}

// RX/TX descriptor structures
#[repr(C, packed)]
struct RxDesc { addr: u64, length: u16, csum: u16, status: u8, errors: u8, special: u16 }
#[repr(C, packed)]
struct TxDesc { addr: u64, length: u16, cso: u8, cmd: u8, status: u8, css: u8, special: u16 }

const RX_RING_SIZE: usize = 32;
const TX_RING_SIZE: usize = 32;

#[repr(align(16))]
struct Aligned<T>(T);

static mut RX_RING: Aligned<[RxDesc; RX_RING_SIZE]> = Aligned([RxDesc{addr:0,length:0,csum:0,status:0,errors:0,special:0}; RX_RING_SIZE]);
static mut TX_RING: Aligned<[TxDesc; TX_RING_SIZE]> = Aligned([TxDesc{addr:0,length:0,cso:0,cmd:0,status:0,css:0,special:0}; TX_RING_SIZE]);
static mut RX_BUFS: Aligned<[[u8; 2048]; RX_RING_SIZE]> = Aligned([[0u8;2048]; RX_RING_SIZE]);
static mut TX_BUFS: Aligned<[[u8; 2048]; TX_RING_SIZE]> = Aligned([[0u8;2048]; TX_RING_SIZE]);

lazy_static! { pub static ref NIC: Mutex<Option<E1000>> = Mutex::new(None); }

impl E1000 {
    pub fn init() -> Option<Self> {
        let mut found = None;
        pci::enumerate(|dev| {
            if dev.vendor == 0x8086 && dev.class == 0x02 { // Intel network
                // BAR0
                let bar0 = pci::read_u32(dev.bus, dev.slot, dev.func, 0x10);
                let mmio_base = (bar0 & 0xFFFF_FFF0) as usize as *mut u32;
                // Enable bus mastering
                let cmd = pci::read_u32(dev.bus, dev.slot, dev.func, 0x04);
                pci::write_u32(dev.bus, dev.slot, dev.func, 0x04, cmd | 0x0000_0004);
                unsafe {
                    // Read MAC from RAL/RAH (0x5400/0x5404)
                    let ral = read_volatile(mmio_base.add(0x5400/4));
                    let rah = read_volatile(mmio_base.add(0x5404/4));
                    let mac = [
                        (ral & 0xFF) as u8,
                        ((ral >> 8) & 0xFF) as u8,
                        ((ral >> 16) & 0xFF) as u8,
                        ((ral >> 24) & 0xFF) as u8,
                        (rah & 0xFF) as u8,
                        ((rah >> 8) & 0xFF) as u8,
                    ];
                    let mut dev = E1000{ regs: mmio_base, mac };
                    dev.init_rings();
                    found = Some(dev);
                }
            }
        });
        if let Some(dev) = found {
            *NIC.lock() = Some(dev);
        }
        found
    }

    unsafe fn write_reg(&self, off: usize, val: u32) { write_volatile(self.regs.add(off/4), val); }
    unsafe fn read_reg(&self, off: usize) -> u32 { read_volatile(self.regs.add(off/4)) }

    fn init_rings(&mut self) {
        unsafe {
            // Setup RX ring
            for i in 0..RX_RING_SIZE {
                RX_RING.0[i].addr = (&RX_BUFS.0[i]) as *const u8 as u64;
                RX_RING.0[i].status = 0;
            }
            self.write_reg(0x02800, (&RX_RING.0 as *const _ as u64) as u32); // RDBAL
            self.write_reg(0x02804, ((&RX_RING.0 as *const _ as u64) >> 32) as u32); // RDBAH
            self.write_reg(0x02808, (RX_RING_SIZE * core::mem::size_of::<RxDesc>()) as u32); // RDLEN
            self.write_reg(0x02810, 0); // RDH
            self.write_reg(0x02818, (RX_RING_SIZE-1) as u32); // RDT
            // Enable receiver
            let rctl = 0x00000002 | 0x00000010 | 0x00000004; // EN | UPE | MPE
            self.write_reg(0x00100, rctl);

            // Setup TX ring
            for i in 0..TX_RING_SIZE {
                TX_RING.0[i].addr = (&TX_BUFS.0[i]) as *const u8 as u64;
                TX_RING.0[i].status = 0;
            }
            self.write_reg(0x03800, (&TX_RING.0 as *const _ as u64) as u32); // TDBAL
            self.write_reg(0x03804, ((&TX_RING.0 as *const _ as u64) >> 32) as u32); // TDBAH
            self.write_reg(0x03808, (TX_RING_SIZE * core::mem::size_of::<TxDesc>()) as u32); // TDLEN
            self.write_reg(0x03810, 0); // TDH
            self.write_reg(0x03818, 0); // TDT
            // Enable transmitter
            let tctl = 0x00000002 | (0x40 << 12) | (0x0F << 4); // EN | CT | COLD
            self.write_reg(0x00400, tctl);
        }
    }

    pub fn poll_rx<F: FnMut(&[u8])>(&self, mut f: F) {
        unsafe {
            let mut rdh = self.read_reg(0x02810) as usize;
            let mut rdt = self.read_reg(0x02818) as usize;
            let mut idx = (rdt + 1) % RX_RING_SIZE;
            while idx != rdh {
                let desc = &mut RX_RING.0[idx];
                if desc.status & 0x01 == 0 { break; }
                let len = desc.length as usize;
                let data = core::slice::from_raw_parts(RX_BUFS.0[idx].as_ptr(), len);
                f(data);
                desc.status = 0;
                rdt = idx; idx = (idx + 1) % RX_RING_SIZE;
            }
            self.write_reg(0x02818, rdt as u32);
        }
    }

    pub fn send(&self, data: &[u8]) -> bool {
        unsafe {
            let tdt = self.read_reg(0x03818) as usize;
            let next = (tdt + 1) % TX_RING_SIZE;
            let desc = &mut TX_RING.0[next];
            let buf = &mut TX_BUFS.0[next];
            if data.len() > buf.len() { return false; }
            buf[..data.len()].copy_from_slice(data);
            desc.length = data.len() as u16;
            desc.cmd = 0x09; // EOP | IFCS
            desc.status = 0;
            self.write_reg(0x03818, next as u32);
            true
        }
    }
}
