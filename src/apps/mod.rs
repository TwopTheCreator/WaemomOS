pub mod settings;
pub mod taskmgr;
pub mod web;
pub mod broom;
pub mod network;

pub fn notes() -> &'static str {
    "Notes\n- Welcome to waemom\n- Demo UI with dock, topbar, and animated windows\n"
}

pub fn files(list: &[alloc::string::String]) -> alloc::string::String {
    use alloc::string::String;
    let mut s = String::from("Files\n");
    for name in list { s.push_str("- "); s.push_str(name); s.push('\n'); }
    s
}

pub fn about() -> &'static str {
    "About waemom\nA tiny hobby OS kernel demo.\n"
}
