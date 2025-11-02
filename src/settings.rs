use lazy_static::lazy_static;
use spin::Mutex;
use alloc::string::String;

#[derive(Clone, Debug)]
pub struct Settings {
    pub elf_enabled: bool,
    pub elf_max_bytes: usize,
    pub linux_mode: bool,
    pub web_enabled: bool,
    // UI setup
    pub language: alloc::string::String,
    pub font_scale: usize,
    pub icons_enabled: bool,
    pub animations_enabled: bool,
    pub loading_animations: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            elf_enabled: true,
            elf_max_bytes: 4096,
            linux_mode: true,
            web_enabled: true,
            language: alloc::string::String::from("en-US"),
            font_scale: 1,
            icons_enabled: true,
            animations_enabled: true,
            loading_animations: true,
        }
    }
}

lazy_static! {
    pub static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
}

pub fn load_from_lock(contents: &str) {
    let mut curr = String::new();
    let mut s = SETTINGS.lock().clone();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if line.starts_with('[') && line.ends_with(']') {
            curr = line.trim_matches(&['[',']'][..]).to_string();
            continue;
        }
        if let Some(eq) = line.find('=') {
            let key = line[..eq].trim();
            let val = line[eq+1..].trim();
            match (curr.as_str(), key) {
                ("elf", "enabled") => s.elf_enabled = parse_bool(val, s.elf_enabled),
                ("elf", "max_bytes") => s.elf_max_bytes = val.parse().unwrap_or(s.elf_max_bytes),
                ("linux", "mode") => s.linux_mode = parse_bool(val, s.linux_mode),
                ("web", "enabled") => s.web_enabled = parse_bool(val, s.web_enabled),
                ("ui", "language") => s.language = val.trim_matches('"').to_string(),
                ("ui", "font_scale") => s.font_scale = val.parse().unwrap_or(s.font_scale),
                ("ui", "icons") => s.icons_enabled = parse_bool(val, s.icons_enabled),
                ("ui", "animations") => s.animations_enabled = parse_bool(val, s.animations_enabled),
                ("ui", "loading_animations") => s.loading_animations = parse_bool(val, s.loading_animations),
                _ => {}
            }
        }
    }
    *SETTINGS.lock() = s;
}

fn parse_bool(v: &str, default: bool) -> bool {
    match v.to_ascii_lowercase().as_str() {
        "true"|"1"|"yes"|"on" => true,
        "false"|"0"|"no"|"off" => false,
        _ => default,
    }
}

pub fn current() -> Settings { SETTINGS.lock().clone() }
