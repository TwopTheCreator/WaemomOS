use core::arch::asm;

pub fn sys_uptime_secs() -> u64 {
    // In lack of user mode, this can be called directly; syscall path provided for future
    crate::pit::uptime_secs()
}

core::arch::global_asm!(r#"
.global __syscall_entry_trampoline
__syscall_entry_trampoline:
    // On entry: RAX=nr, RDI,RSI,RDX,R10,R8,R9 args; RCX=user RIP, R11=user RFLAGS
    push rbp
    mov rbp, rsp
    // Save RCX/R11
    push rcx
    push r11
    // Move R10 into RCX for calling convention; we'll pass 6 args
    mov rcx, r10
    // Call Rust handler: u64 handle_syscall(u64 nr, u64 a1, u64 a2, u64 a3, u64 a4, u64 a5, u64 a6)
    // Arg regs: RDI,RSI,RDX,RCX,R8,R9 ; we already have them
    mov rdi, rax
    call handle_syscall
    // Return value in RAX
    // Restore RCX(user RIP) / R11(RFLAGS)
    pop r11
    pop rcx
    leave
    sysretq
"#);

#[no_mangle]
pub extern "C" fn handle_syscall(nr: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64) -> u64 {
    match nr {
        0 => sys_write(a1 as u64, a2 as *const u8, a3 as usize) as u64,
        1 => { crate::scheduler::sleep_current(a1 as u64); 0 }
        2 => { /* exit */ 0 }
        3 => { // spawn_user_elf(path_ptr, len)
            let s = unsafe { core::slice::from_raw_parts(a1 as *const u8, a2 as usize) };
            if let Ok(p) = core::str::from_utf8(s) { crate::syscalls::spawn_user_elf(p).ok().unwrap_or(0) as u64 } else { u64::MAX }
        }
        _ => u64::MAX,
    }
}

fn sys_write(fd: u64, buf: *const u8, len: usize) -> isize {
    if fd == 1 {
        unsafe {
            let s = core::slice::from_raw_parts(buf, len);
            if let Ok(text) = core::str::from_utf8(s) { crate::console::println(text); return len as isize; }
        }
    }
    -1
}

pub fn handle() {
    // int 0x80 path (legacy) — keep stub
}

pub fn spawn(name: &str) -> u64 {
    extern "C" fn kthread_demo() -> ! { loop { core::hint::spin_loop(); } }
    crate::scheduler::spawn_kernel(name, kthread_demo)
}

pub fn exit() { /* TODO mark current as Zombie */ }

pub fn sleep_ticks(ticks: u64) { crate::scheduler::sleep_current(ticks); }

pub unsafe fn enter_user(rip: u64, rsp: u64) -> ! {
    // Enter ring3 via iretq using user selectors from GDT
    let ucs = crate::gdt::GDT.1.ucode; // user code selector
    let uds = crate::gdt::GDT.1.udata; // user data selector
    let rflags: u64 = 0x202; // IF set
    core::arch::asm!(
        "push rax",
        "push rbx",
        "mov rax, {uds_sel}",
        "push rax",            // SS
        "push rsi",            // RSP
        "push {rflags}",       // RFLAGS
        "mov rbx, {ucs_sel}",
        "push rbx",            // CS
        "push rdi",            // RIP
        "iretq",
        uds_sel = in(reg) ((uds.0 as u64) | 3),
        ucs_sel = in(reg) ((ucs.0 as u64) | 3),
        rflags = in(reg) rflags,
        in("rdi") rip,
        in("rsi") rsp,
        options(noreturn)
    );
}

pub fn spawn_user_elf(path: &str) -> Result<u64, ()> {
    // Load from RAMFS
    let bytes = crate::fs::read(path).map_err(|_| ())?;
    let img = crate::elfloader::parse_elf(&bytes).ok_or(())?;
    let cr3 = crate::mm::alloc_user_space().ok_or(())?;
    // Map ELF into new address space
    if !crate::elfloader::map_into_cr3(cr3, &img) { return Err(()); }
    // Map a user stack and set RSP
    let user_stack_top = 0x0000_7fff_ffff_f000u64;
    let _ = crate::mm::map_user_stack(cr3, user_stack_top, 8).ok_or(())?;
    // Create task that enters user
    let pid = crate::scheduler::spawn_kernel("user", super::scheduler::user_trampoline);
    // Patch its user ctx
    {
        let mut tasks = crate::task::TASKS.lock();
        if let Some(t) = tasks.iter_mut().find(|t| t.pid == pid) {
            t.user = Some(crate::task::UserCtx{ rip: img.entry, rsp: user_stack_top });
            t.cr3 = cr3;
        }
    }
    Ok(pid)
}
