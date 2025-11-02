use bootloader_api::BootInfo;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::paging::{PageTable, PhysFrame, Size4KiB, FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags as PTF, MapperAllSizes};
use x86_64::{PhysAddr, VirtAddr};

static mut BOOT_INFO: Option<&'static BootInfo> = None;
static mut PHYS_OFFSET: VirtAddr = VirtAddr::zero();

pub fn init(boot_info: &'static BootInfo) {
    unsafe { BOOT_INFO = Some(boot_info); }
    init_offset_page_table(boot_info);
    init_frame_alloc(boot_info);
}

static mut MAPPER: Option<OffsetPageTable<'static>> = None;

fn init_offset_page_table(boot_info: &'static BootInfo) {
    let phys_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap_or(0));
    unsafe {
        PHYS_OFFSET = phys_offset;
        let level_4_table = active_level_4_table(phys_offset);
        MAPPER = Some(OffsetPageTable::new(level_4_table, phys_offset));
    }
}

unsafe fn active_level_4_table(phys_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (frame, _) = Cr3::read();
    let phys = frame.start_address();
    let virt = phys_offset + phys.as_u64();
    let table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *table_ptr
}

pub struct BootInfoFrameAlloc {
    next: usize,
    frames: heapless::Vec<PhysFrame, 4096>,
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        if self.next >= self.frames.len() { return None; }
        let f = self.frames[self.next]; self.next += 1; Some(f)
    }
}

pub static ref_frame_alloc: () = ();
lazy_static! { pub static ref FRAME_ALLOC: Mutex<Option<BootInfoFrameAlloc>> = Mutex::new(None); }

fn init_frame_alloc(boot_info: &'static BootInfo) {
    use bootloader_api::info::MemoryRegionKind;
    let mut frames = heapless::Vec::<PhysFrame, 4096>::new();
    if let Some(map) = boot_info.memory_regions.as_ref() {
        for r in map.iter() {
            if r.kind == MemoryRegionKind::Usable {
                let start = r.start as u64; let end = r.end as u64;
                let mut addr = start;
                while addr + 4096 <= end { if let Some(f) = PhysFrame::from_start_address(PhysAddr::new(addr)).ok() { let _ = frames.push(f);} addr += 4096; }
            }
        }
    }
    *FRAME_ALLOC.lock() = Some(BootInfoFrameAlloc{ next: 0, frames });
}

pub fn mapper<'a>() -> Option<&'a mut OffsetPageTable<'a>> {
    unsafe { MAPPER.as_mut() }
}

pub fn with_mapper_for_cr3<T>(cr3: u64, f: impl FnOnce(&mut OffsetPageTable<'_>) -> T) -> Option<T> {
    // Build an OffsetPageTable reference to the given PML4
    let pml4_pa = PhysAddr::new(cr3);
    let pml4_va = phys_to_virt(pml4_pa);
    let lvl4: &mut PageTable = unsafe { &mut *(pml4_va.as_mut_ptr()) };
    let mut mapper = unsafe { OffsetPageTable::new(lvl4, PHYS_OFFSET) };
    Some(f(&mut mapper))
}

pub fn phys_to_virt(pa: PhysAddr) -> VirtAddr { unsafe { PHYS_OFFSET + pa.as_u64() } }

pub fn create_user_pml4() -> Option<u64> {
    // Allocate new PML4 and copy kernel higher-half entries
    let mut alloc = FRAME_ALLOC.lock();
    let fa = alloc.as_mut()?;
    let pml4_frame = fa.allocate_frame()?;
    let pml4_va = phys_to_virt(pml4_frame.start_address());
    let new_pml4 = unsafe { &mut *(pml4_va.as_mut_ptr::<PageTable>()) };
    // zero
    for e in new_pml4.iter_mut() { *e = x86_64::structures::paging::PageTableEntry::new(); }
    // copy upper half from current
    unsafe {
        use x86_64::registers::control::Cr3;
        let (cur, _) = Cr3::read();
        let cur_pml4 = &*(phys_to_virt(cur.start_address()).as_ptr::<PageTable>());
        for i in 256..512 { new_pml4[i] = cur_pml4[i].clone(); }
    }
    Some(pml4_frame.start_address().as_u64())
}

pub fn map_user_region(cr3: u64, vaddr: u64, len: usize, writable: bool) -> Option<Vec<PhysFrame>> {
    let mut frames = Vec::new();
    let flags = PTF::PRESENT | PTF::USER_ACCESSIBLE | if writable { PTF::WRITABLE } else { PTF::empty() };
    let mut alloc = FRAME_ALLOC.lock();
    let fa = alloc.as_mut()?;
    with_mapper_for_cr3(cr3, |mapper| {
        let mut off = 0;
        while off < len {
            let frame = fa.allocate_frame().ok_or(())?;
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(vaddr + off as u64));
            unsafe { mapper.map_to(page, frame, flags, fa).map_err(|_| ())?.flush(); }
            frames.push(frame);
            off += 4096;
        }
        Ok(())
    })?;
    Some(frames)
}

pub fn map_user_stack(cr3: u64, top: u64, pages: usize) -> Option<u64> {
    let size = pages * 4096;
    let base = top - size as u64;
    let _ = map_user_region(cr3, base, size, true)?;
    Some(top)
}

pub fn alloc_user_space() -> Option<u64> {
    create_user_pml4()
}
