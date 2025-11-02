use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
}

pub fn on_scancode(scancode: u8) {
    let mut kbd = KEYBOARD.lock();
    if let Ok(Some(event)) = kbd.add_byte(scancode) {
        if let Some(key) = kbd.process_keyevent(event) {
            match key {
                DecodedKey::Unicode(c) => crate::tty::write_char(c),
                DecodedKey::RawKey(_) => {},
            }
        }
    }
}
