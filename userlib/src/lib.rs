#![no_std]

pub fn spawn(_name: &str) -> u64 { 0 }
pub fn exit() -> ! { loop {} }
pub fn sleep(_ticks: u64) {}

pub mod net {
    pub fn socket_udp(_port: u16) -> i32 { 0 }
    pub fn send(_sock: i32, _buf: &[u8]) -> isize { _buf.len() as isize }
    pub fn recv(_sock: i32, _buf: &mut [u8]) -> isize { 0 }
}
