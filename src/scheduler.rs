use crate::task::{TASKS, Task, State, alloc_pid};
use crate::context::{self, Context};
use crate::pit;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref CURRENT: Mutex<Option<usize>> = Mutex::new(None);
    static ref READY: Mutex<alloc::collections::VecDeque<usize>> = Mutex::new(alloc::collections::VecDeque::new());
}

pub fn init() {
    // spawn an idle task
    let idle_pid = alloc_pid();
    let idle = Task::new_kernel(idle_pid, "idle", idle_task);
    let mut tasks = TASKS.lock();
    tasks.push(idle);
    READY.lock().push_back(0);
    *CURRENT.lock() = Some(0);
}

extern "C" fn idle_task() -> ! { loop { core::hint::spin_loop(); } }

extern "C" fn user_trampoline() -> ! {
    // Enter user mode via sysret using current task stored rip/rsp
    if let Some(mut t) = current_task_mut() {
        if let Some(uc) = t.user.take() {
            unsafe { super::syscalls::enter_user(uc.rip, uc.rsp); }
        }
    }
    loop { core::hint::spin_loop(); }
}

pub fn on_tick() {
    // wake sleeping tasks
    let now = pit::uptime_secs();
    let mut tasks = TASKS.lock();
    for (i, t) in tasks.iter_mut().enumerate() {
        if let State::Sleeping(until) = t.state { if now >= until { t.state = State::Ready; READY.lock().push_back(i); } }
    }
    // preempt current
    if let Some(cur) = *CURRENT.lock() {
        if matches!(tasks[cur].state, State::Running | State::Ready) {
            tasks[cur].state = State::Ready;
            READY.lock().push_back(cur);
        }
    }
    // pick next
    if let Some(next) = READY.lock().pop_front() {
        let cur_opt = *CURRENT.lock();
        *CURRENT.lock() = Some(next);
        // prepare states and pointers then drop lock before switching
        let (old_ptr, new_ptr, next_cr3) = if let Some(cur) = cur_opt {
            tasks[next].state = State::Running;
            if matches!(tasks[cur].state, State::Running) { tasks[cur].state = State::Ready; }
            let old_ptr = &mut tasks[cur].ctx as *mut Context;
            let new_ptr = &tasks[next].ctx as *const Context;
            (old_ptr, new_ptr, tasks[next].cr3)
        } else {
            tasks[next].state = State::Running;
            // no current, save into dummy
            (unsafe { &mut crate::context::DUMMY as *mut Context }, &tasks[next].ctx as *const Context, tasks[next].cr3)
        };
        // switch address space if needed
        unsafe {
            use x86_64::registers::control::Cr3;
            let (cur, _) = Cr3::read();
            if cur.start_address().as_u64() != next_cr3 {
                use x86_64::{PhysAddr};
                Cr3::write(x86_64::PhysFrame::containing_address(PhysAddr::new(next_cr3)), x86_64::registers::control::Cr3Flags::empty());
            }
        }
        drop(tasks);
        unsafe { crate::context::switch(old_ptr, new_ptr); }
    }
}

pub fn current_task_mut() -> Option<spin::MutexGuard<'static, alloc::vec::Vec<crate::task::Task>>> { Some(crate::task::TASKS.lock()) }

pub fn spawn_kernel(name: &str, entry: extern "C" fn() -> !) -> u64 {
    let pid = alloc_pid();
    let t = Task::new_kernel(pid, name, entry);
    let mut tasks = TASKS.lock();
    let idx = tasks.len();
    tasks.push(t);
    READY.lock().push_back(idx);
    pid
}

pub fn sleep_current(ticks: u64) {
    let until = pit::uptime_secs().saturating_add(ticks / 100);
    let mut tasks = TASKS.lock();
    if let Some(cur) = *CURRENT.lock() { tasks[cur].state = State::Sleeping(until); }
}
