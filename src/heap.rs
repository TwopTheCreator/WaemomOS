use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;

struct Bump {
    start: usize,
    next: usize,
    end: usize,
    inited: bool,
}

impl Bump {
    const fn new() -> Self { Self { start: 0, next: 0, end: 0, inited: false } }
    fn init(&mut self) {
        if self.inited { return; }
        let start = unsafe { HEAP.as_ptr() as usize };
        self.start = start;
        self.next = start;
        self.end = start + HEAP.len();
        self.inited = true;
    }
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if !self.inited { self.init(); }
        let align_mask = layout.align() - 1;
        let aligned = (self.next + align_mask) & !align_mask;
        let end = aligned.saturating_add(layout.size());
        if end > self.end { return null_mut(); }
        self.next = end;
        aligned as *mut u8
    }
}

#[global_allocator]
static GLOBAL: WaemomAlloc = WaemomAlloc;

struct WaemomAlloc;

static BUMP: Mutex<Bump> = Mutex::new(Bump::new());

unsafe impl GlobalAlloc for WaemomAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        BUMP.lock().alloc(layout)
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // no-op (leaking)
    }
}

#[link_section = ".bss.heap"]
static mut HEAP: [u8; 512 * 1024] = [0; 512 * 1024];

pub fn init() { BUMP.lock().init(); }
