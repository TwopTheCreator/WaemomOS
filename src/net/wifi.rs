use alloc::string::String;
use alloc::vec::Vec;
use super::{set_ssid, set_ip, enable_encryption};
use lazy_static::lazy_static;
use spin::Mutex;

#[derive(Clone)]
pub struct Network { pub ssid: &'static str, pub strength: u8 }

lazy_static! {
    static ref CURRENT: Mutex<Option<&'static str>> = Mutex::new(None);
}

pub fn scan() -> Vec<Network> {
    vec![
        Network{ ssid: "waemom-net", strength: 4 },
        Network{ ssid: "guest", strength: 2 },
        Network{ ssid: "lab", strength: 3 },
    ]
}

pub fn connect(ssid: &str, _password: &str) -> bool {
    *CURRENT.lock() = Some(Box::leak(ssid.to_string().into_boxed_str()));
    set_ssid(ssid);
    super::ip::dhcp();
    enable_encryption(true);
    true
}

pub fn current_ssid() -> Option<&'static str> { *CURRENT.lock() }
