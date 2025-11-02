use alloc::string::String;
use crate::settings::Settings;

pub fn view(settings: &Settings) -> String {
    let mut s = String::from("System Settings\n\n");
    s.push_str(&format!("ELF inspector: {}\n", if settings.elf_enabled {"enabled"} else {"disabled"}));
    s.push_str(&format!("ELF max bytes: {}\n", settings.elf_max_bytes));
    s.push_str(&format!("Linux mode: {}\n", if settings.linux_mode {"on"} else {"off"}));
    s.push_str(&format!("Web viewer: {}\n", if settings.web_enabled {"on"} else {"off"}));
    s.push_str("\nUI\n");
    s.push_str(&format!("Language: {}\n", settings.language));
    s.push_str(&format!("Font scale: {}\n", settings.font_scale));
    s.push_str(&format!("Icons: {}\n", if settings.icons_enabled {"on"} else {"off"}));
    s.push_str(&format!("Animations: {}\n", if settings.animations_enabled {"on"} else {"off"}));
    s.push_str(&format!("Loading animations: {}\n", if settings.loading_animations {"on"} else {"off"}));
    s
}
