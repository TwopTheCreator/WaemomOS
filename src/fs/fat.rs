pub struct Bpb {
    pub bps: u16,
    pub spc: u8,
    pub rsv: u32,
    pub nfats: u8,
    pub fatsz: u32,
    pub root_entries: u32,
    pub total_sectors: u32,
    pub fat_start: u32,
    pub root_dir_sectors: u32,
    pub first_data: u32,
    pub fat_bits: u8,
}

pub fn read_bpb(dev: &mut dyn crate::block::BlockDevice) -> Option<Bpb> {
    let mut buf = [0u8;512];
    if !dev.read_sector(0, &mut buf) { return None; }
    if &buf[510..512] != [0x55,0xAA] { return None; }
    let bps = u16::from_le_bytes([buf[11],buf[12]]);
    let spc = buf[13];
    let rsv = u16::from_le_bytes([buf[14],buf[15]]) as u32;
    let nfats = buf[16];
    let root_entries = u16::from_le_bytes([buf[17],buf[18]]) as u32;
    let total16 = u16::from_le_bytes([buf[19],buf[20]]) as u32;
    let total = if total16!=0 { total16 } else { u32::from_le_bytes([buf[32],buf[33],buf[34],buf[35]]) };
    let fatsz16 = u16::from_le_bytes([buf[22],buf[23]]) as u32;
    let fatsz = if fatsz16 != 0 { fatsz16 } else { u32::from_le_bytes([buf[36],buf[37],buf[38],buf[39]]) };
    let fat_start = rsv;
    let root_dir_sectors = ((root_entries * 32) + (bps as u32 -1)) / bps as u32;
    let first_data = rsv + (nfats as u32 * fatsz) + root_dir_sectors;
    let fat_bits = if fatsz16 != 0 { 16 } else { 32 };
    Some(Bpb{ bps, spc, rsv, nfats, fatsz, root_entries, total_sectors: total, fat_start, root_dir_sectors, first_data, fat_bits })
}

pub fn list_root(dev: &mut dyn crate::block::BlockDevice) -> heapless::Vec<heapless::String<32>, 128> {
    let mut out = heapless::Vec::new();
    if let Some(bpb) = read_bpb(dev) {
        let bps = bpb.bps;
        let root_dir_lba = bpb.rsv + (bpb.nfats as u32 * bpb.fatsz);
        // assume FAT16 fixed root dir size from BPB
        let mut buf = [0u8;512];
        let mut lba = root_dir_lba;
        for _ in 0..32 { // scan 32 sectors of root
            if !dev.read_sector(lba, &mut buf) { break; }
            for i in (0..512).step_by(32) {
                let name = &buf[i..i+11];
                let attr = buf[i+11];
                if name[0] == 0x00 { break; }
                if name[0] == 0xE5 || attr == 0x0F { continue; } // deleted or LFN
                let mut s = heapless::String::<32>::new();
                for &b in &name[0..8] { if b != b' ' { let _ = s.push(b as char); } }
                if name[8] != b' ' { let _ = s.push('.'); for &b in &name[8..11] { if b != b' ' { let _ = s.push(b as char); } } }
                let _ = out.push(s);
            }
            lba += 1;
        }
    }
    out
}

pub fn read_file_root_8_3(dev: &mut dyn crate::block::BlockDevice, name83: &str) -> Option<alloc::vec::Vec<u8>> {
    let bpb = read_bpb(dev)?;
    let bps = bpb.bps;
    let first_data = bpb.first_data;
    let root_dir_lba = bpb.rsv + (bpb.nfats as u32 * bpb.fatsz);
    let mut buf = [0u8;512];
    let mut lba = root_dir_lba;
    for _ in 0..32 {
        if !dev.read_sector(lba, &mut buf) { break; }
        for i in (0..512).step_by(32) {
            let name = &buf[i..i+11];
            let attr = buf[i+11];
            if name[0] == 0x00 { break; }
            if name[0] == 0xE5 || attr == 0x0F { continue; }
            let mut s = heapless::String::<12>::new();
            for &b in &name[0..8] { if b != b' ' { let _ = s.push(b as char); } }
            if name[8] != b' ' { let _ = s.push('.'); for &b in &name[8..11] { if b != b' ' { let _ = s.push(b as char); } } }
            if s.as_str().eq_ignore_ascii_case(name83) {
                let lo = u16::from_le_bytes([buf[i+26],buf[i+27]]) as u32;
                let hi = u16::from_le_bytes([buf[i+20],buf[i+21]]) as u32;
                let first_cluster = (hi<<16)|lo;
                let size = u32::from_le_bytes([buf[i+28],buf[i+29],buf[i+30],buf[i+31]]) as usize;
                return read_chain_fat16(dev, bps, first_data, first_cluster, size);
            }
        }
        lba += 1;
    }
    None
}

fn read_chain_fat16(dev: &mut dyn crate::block::BlockDevice, bps: u16, first_data: u32, mut cluster: u32, size: usize) -> Option<alloc::vec::Vec<u8>> {
    // Minimal FAT16 cluster chain reader with FAT traversal
    let spc = 1u32; // assumes 1 sector per cluster for small volumes; refine using BPB.spc as needed
    let fat_start = read_bpb(dev)?.fat_start;
    let mut out = alloc::vec::Vec::with_capacity(size);
    let first_cluster = cluster;
    let mut remaining = size;
    let mut buf = [0u8;512];
    while cluster >= 2 && cluster < 0xFFF8 {
        let lba = first_data + (cluster - 2) * spc;
        for s in 0..spc {
            if !dev.read_sector(lba + s, &mut buf) { return None; }
            let to_copy = core::cmp::min(512usize, remaining);
            out.extend_from_slice(&buf[..to_copy]).ok()?;
            remaining -= to_copy;
            if remaining == 0 { break; }
        }
        // Read next cluster from FAT table
        let fat_off_bytes = cluster * 2;
        let fat_sector = fat_start + (fat_off_bytes / bps as u32);
        let ent_off = (fat_off_bytes % bps as u32) as usize;
        if !dev.read_sector(fat_sector, &mut buf) { return None; }
        let entry = u16::from_le_bytes([buf[ent_off], buf[ent_off+1]]) as u32;
        if entry >= 0xFFF8 { break; }
        cluster = entry;
    }
    Some(out)
}
