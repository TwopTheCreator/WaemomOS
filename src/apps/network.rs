use alloc::string::String;
use crate::{net, window};

pub fn view() -> String {
    let mut s = String::from("Network\n\n");
    s.push_str(&format!("SSID: {}\n", net::ssid().unwrap_or("<not connected>".into())));
    s.push_str(&format!("IP: {}\n", net::ip().map(ip_to_string).unwrap_or("0.0.0.0".into())));
    s.push_str(&format!("Encryption: {}\n", if net::is_encrypted(){"on"} else {"off"}));
    if let Some(mac) = crate::net::e1000::E1000::init().as_ref().map(|e| e.mac) {
        s.push_str(&format!("e1000 MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n", mac[0],mac[1],mac[2],mac[3],mac[4],mac[5]));
    }
    s.push_str("\nAvailable Wi‑Fi:\n");
    for n in net::wifi::scan() { s.push_str(&format!("- {} ({} bars)\n", n.ssid, n.strength)); }
    s
}

pub fn connect_default() {
    let _ = net::wifi::connect("waemom-net", "password");
    // Start UDP echo on port 7 (echo)
    let _ = crate::net::netstack::open_udp(7);
}

fn ip_to_string(ip: [u8;4]) -> String { alloc::format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]) }
