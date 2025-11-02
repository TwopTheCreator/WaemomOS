pub trait BlockDevice {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8;512]) -> bool;
}

pub struct AtaDevice;
impl AtaDevice {
    pub fn new() -> Self { Self }
}
impl BlockDevice for AtaDevice {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8;512]) -> bool {
        let mut words = [0u16;256];
        if crate::ata::read_sector_lba28(lba, &mut words) {
            for i in 0..256 { let b = words[i].to_le_bytes(); buf[i*2]=b[0]; buf[i*2+1]=b[1]; }
            true
        } else { false }
    }
}
