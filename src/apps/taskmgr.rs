use alloc::string::String;

#[derive(Clone)]
pub struct TaskInfo {
    pub pid: u32,
    pub name: String,
    pub state: &'static str,
}

pub fn view(tasks: &[TaskInfo]) -> String {
    let mut s = String::from("Task Manager\n\nPID    NAME         STATE\n");
    for t in tasks {
        s.push_str(&format!("{:<6} {:<12} {}\n", t.pid, t.name, t.state));
    }
    s
}
