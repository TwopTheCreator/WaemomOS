use lazy_static::lazy_static;
use spin::Mutex;
use crate::image::{self, Image};

lazy_static! {
    static ref NOTES: Mutex<Option<Image>> = Mutex::new(None);
    static ref FILES: Mutex<Option<Image>> = Mutex::new(None);
    static ref ABOUT: Mutex<Option<Image>> = Mutex::new(None);
    static ref SETTINGS: Mutex<Option<Image>> = Mutex::new(None);
    static ref TASK: Mutex<Option<Image>> = Mutex::new(None);
    static ref BROOM: Mutex<Option<Image>> = Mutex::new(None);
}

fn parse_or_set(slot: &Mutex<Option<Image>>, bytes: &[u8]) -> Option<&'static Image> {
    if slot.lock().is_none() {
        if let Some(img) = image::parse_ppm_p3(bytes) { *slot.lock() = Some(img); }
    }
    // extend lifetime by leaking; safe here for demo OS
    if slot.lock().is_some() { Some(Box::leak(Box::new(slot.lock().clone().unwrap()))) } else { None }
}

pub fn icon_notes() -> Option<&'static Image> { parse_or_set(&NOTES, include_bytes!("../../../assets/icons/notes.ppm")) }
pub fn icon_files() -> Option<&'static Image> { parse_or_set(&FILES, include_bytes!("../../../assets/icons/files.ppm")) }
pub fn icon_about() -> Option<&'static Image> { parse_or_set(&ABOUT, include_bytes!("../../../assets/icons/about.ppm")) }
pub fn icon_settings() -> Option<&'static Image> { parse_or_set(&SETTINGS, include_bytes!("../../../assets/icons/settings.ppm")) }
pub fn icon_task() -> Option<&'static Image> { parse_or_set(&TASK, include_bytes!("../../../assets/icons/task.ppm")) }
pub fn icon_broom() -> Option<&'static Image> { parse_or_set(&BROOM, include_bytes!("../../../assets/icons/broom.ppm")) }
