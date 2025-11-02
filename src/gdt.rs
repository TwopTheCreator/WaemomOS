use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub(crate) use GDT;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        static mut STACK: [u8; 4096] = [0; 4096];
        let stack_top = VirtAddr::from_ptr(unsafe { &STACK }) + 4096;
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_top;
        tss
    };

    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kcode = gdt.add_entry(Descriptor::kernel_code_segment());
        let kdata = gdt.add_entry(Descriptor::kernel_data_segment());
        let ucode = gdt.add_entry(Descriptor::user_code_segment());
        let udata = gdt.add_entry(Descriptor::user_data_segment());
        let tss_sel = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { kcode, kdata, ucode, udata, tss_selector: tss_sel })
    };
}

pub struct Selectors { pub kcode: SegmentSelector, pub kdata: SegmentSelector, pub ucode: SegmentSelector, pub udata: SegmentSelector, pub tss_selector: SegmentSelector }

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, SS};
    use x86_64::instructions::tables::load_tss;
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kcode);
        SS::set_reg(GDT.1.kdata);
        DS::set_reg(GDT.1.kdata);
        ES::set_reg(GDT.1.kdata);
        load_tss(GDT.1.tss_selector);
    }
    init_syscall();
}

fn init_syscall() {
    use x86_64::registers::model_specific::{Efer, EferFlags, LStar, Star, SFMask};
    // Enable SYSCALL/SYSRET
    unsafe { Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS); }
    // STAR encodes kernel/user segment selectors
    let kcs = GDT.1.kcode.0 as u64;
    let ucs = GDT.1.ucode.0 as u64 | 3; // RPL3
    let value = ((ucs) << 48) | ((kcs) << 32);
    unsafe { Star::write(value); }
    // Syscall entry point; we will use int 0x80 for now but set LSTAR to our handler stub
    extern "C" { fn __syscall_entry_trampoline(); }
    unsafe { LStar::write(x86_64::VirtAddr::from_ptr(__syscall_entry_trampoline).as_u64()); }
    unsafe { SFMask::write(0); }
}
