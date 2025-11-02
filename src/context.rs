#[repr(C)]
pub struct Context {
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub rbx: u64, pub rbp: u64, pub rsp: u64,
    pub rip: u64, pub rflags: u64,
}

impl Context { pub const fn zero() -> Self { Self { r15:0,r14:0,r13:0,r12:0,rbx:0,rbp:0,rsp:0,rip:0,rflags:0 } } }

pub static mut DUMMY: Context = Context::zero();

#[inline(always)]
pub unsafe fn switch(old: *mut Context, new: *const Context) {
    core::arch::asm!(
        // save callee-saved
        "mov [rdi+0x00], r15",
        "mov [rdi+0x08], r14",
        "mov [rdi+0x10], r13",
        "mov [rdi+0x18], r12",
        "mov [rdi+0x20], rbx",
        "mov [rdi+0x28], rbp",
        "lea rax, [rip]",
        "mov [rdi+0x38], rsp",
        "mov [rdi+0x30], rax",
        // load new
        "mov r15, [rsi+0x00]",
        "mov r14, [rsi+0x08]",
        "mov r13, [rsi+0x10]",
        "mov r12, [rsi+0x18]",
        "mov rbx, [rsi+0x20]",
        "mov rbp, [rsi+0x28]",
        "mov rsp, [rsi+0x38]",
        "jmp [rsi+0x30]",
        in("rdi") old, in("rsi") new,
        options(noreturn)
    );
}
