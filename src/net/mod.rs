pub mod wifi;
pub mod ip;
pub mod crypto;

use lazy_static::lazy_static;
use spin::Mutex;
use alloc::string::String;

#[derive(Clone)]
pub struct NetConfig {
    pub ssid: Option<String>,
    pub ip: Option<[u8;4]>,
    pub enc_enabled: bool,
}

lazy_static! { static ref NET: Mutex<NetConfig> = Mutex::new(NetConfig{ ssid:None, ip:None, enc_enabled:false }); }

pub fn enable_encryption(on: bool) { NET.lock().enc_enabled = on; }
pub fn is_encrypted() -> bool { NET.lock().enc_enabled }

pub fn set_ip(ip: [u8;4]) { NET.lock().ip = Some(ip); }
pub fn ip() -> Option<[u8;4]> { NET.lock().ip }

pub fn set_ssid(s: &str) { NET.lock().ssid = Some(s.to_string()); }
pub fn ssid() -> Option<String> { NET.lock().ssid.clone() }
