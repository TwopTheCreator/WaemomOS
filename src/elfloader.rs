use x86_64::structures::paging::{PageTableFlags as PTF, Mapper, Page, Size4KiB};
use x86_64::{VirtAddr};

pub struct ElfImage<'a> { pub entry: u64, pub segments: heapless::Vec<(&'a [u8], u64), 16> }

pub fn parse_elf<'a>(bytes: &'a [u8]) -> Option<ElfImage<'a>> {
    if bytes.len() < 64 || &bytes[0..4] != b"\x7FELF" || bytes[4] != 2 { return None; }
    let e_entry = u64::from_le_bytes(bytes[24..32].try_into().ok()?);
    let phoff = u64::from_le_bytes(bytes[32..40].try_into().ok()?);
    let phentsize = u16::from_le_bytes(bytes[54..56].try_into().ok()?) as usize;
    let phnum = u16::from_le_bytes(bytes[56..58].try_into().ok()?) as usize;
    let mut segs = heapless::Vec::<(&[u8], u64), 16>::new();
    for i in 0..phnum {
        let off = phoff as usize + i*phentsize;
        let p_type = u32::from_le_bytes(bytes[off..off+4].try_into().ok()?);
        if p_type != 1 { continue; } // PT_LOAD
        let p_offset = u64::from_le_bytes(bytes[off+8..off+16].try_into().ok()?);
        let p_vaddr = u64::from_le_bytes(bytes[off+16..off+24].try_into().ok()?);
        let p_filesz = u64::from_le_bytes(bytes[off+32..off+40].try_into().ok()?);
        let start = p_offset as usize;
        let end = start + p_filesz as usize;
        let _ = segs.push((&bytes[start..end], p_vaddr));
    }
    Some(ElfImage{ entry: e_entry, segments: segs })
}

pub fn map_into_cr3(cr3: u64, img: &ElfImage) -> bool {
    // Allocate frames and copy segment data into them, mapping as USER|WRITABLE
    for (seg, vaddr) in img.segments.iter() {
        let len = seg.len();
        let start = *vaddr as usize;
        if let Some(frames) = crate::mm::map_user_region(cr3, *vaddr, len, true) {
            // copy into physical frames via phys->virt
            let mut copied = 0usize;
            for frame in frames {
                let va = crate::mm::phys_to_virt(frame.start_address());
                let dst = unsafe { core::slice::from_raw_parts_mut(va.as_mut_ptr::<u8>(), 4096) };
                let to_copy = core::cmp::min(4096, len - copied);
                dst[..to_copy].copy_from_slice(&seg[copied..copied+to_copy]);
                if to_copy < 4096 { for b in &mut dst[to_copy..] { *b = 0; } }
                copied += to_copy;
                if copied >= len { break; }
            }
        }
    }
    true
}
