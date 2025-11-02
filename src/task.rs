use crate::context::Context;
use alloc::vec::Vec;
use alloc::alloc;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State { Ready, Running, Sleeping(u64), Zombie }

pub struct Task {
    pub pid: u64,
    pub name: heapless::String<32>,
    pub ctx: Context,
    pub stack_ptr: *mut u8,
    pub cr3: u64, // address space root
    pub user: Option<UserCtx>,
    pub state: State,
    pub priority: u8,
}

pub struct UserCtx { pub rip: u64, pub rsp: u64 }

impl Task {
    pub fn new_kernel(pid: u64, name: &str, entry: extern "C" fn() -> !) -> Self {
        // allocate stack
        let stack_size = 16*1024;
        let layout = core::alloc::Layout::from_size_align(stack_size, 16).unwrap();
        let stack_ptr = unsafe { alloc::alloc::alloc(layout) };
        let sp = unsafe { stack_ptr.add(stack_size) } as u64;
        let mut ctx = Context::zero();
        ctx.rsp = sp;
        ctx.rip = entry as u64;
        let mut n = heapless::String::<32>::new(); let _ = n.push_str(name);
        // inherit current CR3 for now (kernel-only address space)
        let cr3 = unsafe { x86_64::registers::control::Cr3::read().0.start_address().as_u64() };
        Self { pid, name: n, ctx, stack_ptr, cr3, user: None, state: State::Ready, priority: 10 }
    }
}

lazy_static! {
    pub static ref TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());
    static ref NEXT_PID: AtomicU64 = AtomicU64::new(1);
}

pub fn alloc_pid() -> u64 { NEXT_PID.fetch_add(1, Ordering::Relaxed) }
