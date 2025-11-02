use x86_64::instructions::port::Port;

const ATA_REG_DATA: u16 = 0x1F0;
const ATA_REG_SECCOUNT0: u16 = 0x1F2;
const ATA_REG_LBA0: u16 = 0x1F3;
const ATA_REG_LBA1: u16 = 0x1F4;
const ATA_REG_LBA2: u16 = 0x1F5;
const ATA_REG_HDDEVSEL: u16 = 0x1F6;
const ATA_REG_COMMAND: u16 = 0x1F7;
const ATA_REG_STATUS: u16 = 0x1F7;

pub fn read_sector_lba28(lba: u32, buf: &mut [u16;256]) -> bool {
    unsafe {
        let mut status: u8;
        // Select drive and LBA bits 24-27
        let mut hd: Port<u8> = Port::new(ATA_REG_HDDEVSEL);
        hd.write(0xE0 | (((lba >> 24) & 0x0F) as u8));
        Port::<u8>::new(ATA_REG_SECCOUNT0).write(1);
        Port::<u8>::new(ATA_REG_LBA0).write((lba & 0xFF) as u8);
        Port::<u8>::new(ATA_REG_LBA1).write(((lba >> 8) & 0xFF) as u8);
        Port::<u8>::new(ATA_REG_LBA2).write(((lba >> 16) & 0xFF) as u8);
        Port::<u8>::new(ATA_REG_COMMAND).write(0x20); // READ SECTORS
        // Wait for DRQ
        loop {
            status = Port::<u8>::new(ATA_REG_STATUS).read();
            if status & 0x08 != 0 { break; }
            if status & 0x01 != 0 { return false; }
        }
        // Read 256 words
        let mut data: Port<u16> = Port::new(ATA_REG_DATA);
        for i in 0..256 { buf[i] = data.read(); }
        true
    }
}
