use smoltcp::iface::{Config, Interface, SocketSet, SocketHandle};
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address, IpEndpoint};
use smoltcp::phy::{self, DeviceCapabilities, Device};
use smoltcp::socket::{udp, tcp};
use smoltcp::time::Instant;
use lazy_static::lazy_static;
use spin::Mutex;
use alloc::{vec, vec::Vec};

// Loopback-like PHY: tx goes to rx buffer
pub struct LoopPhy { rx: heapless::Vec<heapless::Vec<u8, 1536>, 4> }
impl LoopPhy { pub fn new() -> Self { Self { rx: heapless::Vec::new() } } }

impl<'a> phy::Device<'a> for LoopPhy {
    type RxToken = Rx;
    type TxToken = Tx;
    fn capabilities(&self) -> DeviceCapabilities { DeviceCapabilities::default() }
    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        if let Some(buf) = self.rx.pop() { Some((Rx(buf), Tx(&mut self.rx))) } else { None }
    }
    fn transmit(&'a mut self) -> Option<Self::TxToken> { Some(Tx(&mut self.rx)) }
}

// e1000-backed Device
pub struct E1000Phy;
impl<'a> phy::Device<'a> for E1000Phy {
    type RxToken = RxE1000;
    type TxToken = TxE1000;
    fn capabilities(&self) -> DeviceCapabilities { DeviceCapabilities::default() }
    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let mut pkt: Option<heapless::Vec<u8, 1536>> = None;
        if let Some(ref nic) = *crate::net::e1000::NIC.lock() {
            nic.poll_rx(|data| {
                let mut v = heapless::Vec::new();
                let _ = v.extend_from_slice(data);
                pkt = Some(v);
            });
        }
        pkt.map(|p| (RxE1000(p), TxE1000))
    }
    fn transmit(&'a mut self) -> Option<Self::TxToken> { Some(TxE1000) }
}

pub struct RxE1000(heapless::Vec<u8, 1536>);
pub struct TxE1000;
impl phy::RxToken for RxE1000 {
    fn consume<R, F>(self, _ts: Instant, f: F) -> R where F: FnOnce(&mut [u8]) -> R {
        let mut buf = self.0;
        f(&mut buf)
    }
}
impl phy::TxToken for TxE1000 {
    fn consume<R, F>(self, _ts: Instant, len: usize, f: F) -> R where F: FnOnce(&mut [u8]) -> R {
        let mut buf = [0u8; 1536];
        let r = f(&mut buf[..len]);
        if let Some(ref nic) = *crate::net::e1000::NIC.lock() { let _ = nic.send(&buf[..len]); }
        r
    }
}

pub struct Rx(heapless::Vec<u8, 1536>);
pub struct Tx<'a>(&'a mut heapless::Vec<heapless::Vec<u8, 1536>, 4>);
impl phy::RxToken for Rx {
    fn consume<R, F>(self, _ts: Instant, f: F) -> R where F: FnOnce(&mut [u8]) -> R {
        let mut buf = self.0;
        f(&mut buf)
    }
}
impl<'a> phy::TxToken for Tx<'a> {
    fn consume<R, F>(self, _ts: Instant, len: usize, f: F) -> R where F: FnOnce(&mut [u8]) -> R {
        let mut tmp = [0u8; 1536];
        let r = f(&mut tmp[..len]);
        if let Some(ref nic) = *crate::net::e1000::NIC.lock() { let _ = nic.send(&tmp[..len]); }
        else { let mut v = heapless::Vec::<u8,1536>::new(); let _ = v.extend_from_slice(&tmp[..len]); let _ = self.0.push(v); }
        r
    }
}

pub struct NetStack<'a> {
    pub phy: LoopPhy,
    pub iface: Interface<'a, LoopPhy>,
    pub sockets: SocketSet<'a>,
    pub udp: Option<SocketHandle>,
    pub tcp: Option<SocketHandle>,
}

lazy_static! { pub static ref NET: Mutex<Option<NetStack<'static>>> = Mutex::new(None); }

impl<'a> NetStack<'a> {
    pub fn init(mac: [u8;6], ip: [u8;4]) {
        let mut cfg = Config::new(EthernetAddress(mac).into());
        let mut phy = LoopPhy::new();
        let mut iface = Interface::new(cfg, &mut phy, Instant::from_millis(0));
        iface.update_ip_addrs(|addrs| { let _ = addrs.push(IpCidr::new(Ipv4Address(ip).into(), 24)); });
        let sockets = SocketSet::new([]);
        let stack = NetStack { phy, iface, sockets, udp: None, tcp: None };
        *NET.lock() = Some(unsafe { core::mem::transmute::<NetStack<'_>, NetStack<'static>>(stack) });
        // Open UDP echo
        let _ = open_udp(7);
        // Open TCP echo
        let _ = open_tcp_listen(7);
    }
}
        iface.update_ip_addrs(|addrs| { let _ = addrs.push(IpCidr::new(Ipv4Address(ip).into(), 24)); });
        let sockets = SocketSet::new([]);
        let stack = NetStack { phy, iface, sockets, udp: None, tcp: None };
        *NET.lock() = Some(unsafe { core::mem::transmute::<NetStack<'_>, NetStack<'static>>(stack) });
    }
}

pub fn poll() {
    if let Some(ref mut ns) = *NET.lock() {
        // Feed e1000 RX into loopback buffer
        if let Some(ref nic) = *crate::net::e1000::NIC.lock() {
            nic.poll_rx(|data| {
                let mut v = heapless::Vec::<u8,1536>::new(); let _ = v.extend_from_slice(data); let _ = ns.phy.rx.push(v);
            });
        }
        let _ = ns.iface.poll(Instant::from_millis(crate::pit::uptime_secs() as i64 * 1000), &mut ns.phy, &mut ns.sockets);
        // UDP echo if socket present
        if let Some(h) = ns.udp {
            let sock = ns.sockets.get_mut::<udp::Socket>(h);
            while let Ok((data, ep)) = sock.recv() { let _ = sock.send_slice(data, ep); }
        }
        // TCP echo
        if let Some(h) = ns.tcp {
            let sock = ns.sockets.get_mut::<tcp::Socket>(h);
            if !sock.is_open() { let _ = sock.listen(7); }
            if sock.can_recv() {
                let mut buf = [0u8; 1024];
                if let Ok(n) = sock.recv_slice(&mut buf) { let _ = sock.send_slice(&buf[..n]); }
            }
        }
    }
}

pub fn open_udp(local_port: u16) -> Option<SocketHandle> {
    if let Some(ref mut ns) = *NET.lock() {
        let rx = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 8], vec![0; 2048]);
        let tx = udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 8], vec![0; 2048]);
        let mut sock = udp::Socket::new(rx, tx);
        let _ = sock.bind(IpEndpoint::new(Ipv4Address::UNSPECIFIED.into(), local_port));
        let handle = ns.sockets.add(sock);
        ns.udp = Some(handle);
        return Some(handle);
    }
    None
}

pub fn open_tcp_listen(local_port: u16) -> Option<SocketHandle> {
    if let Some(ref mut ns) = *NET.lock() {
        let rx_buf = tcp::SocketBuffer::new(vec![0; 2048]);
        let tx_buf = tcp::SocketBuffer::new(vec![0; 2048]);
        let mut sock = tcp::Socket::new(rx_buf, tx_buf);
        let _ = sock.listen(local_port);
        let handle = ns.sockets.add(sock);
        ns.tcp = Some(handle);
        return Some(handle);
    }
    None
}

pub fn udp_send(handle: SocketHandle, dst: (Ipv4Address, u16), data: &[u8]) -> bool {
    if let Some(ref mut ns) = *NET.lock() {
        let sock = ns.sockets.get_mut::<udp::Socket>(handle);
        let ep = IpEndpoint::new(dst.0.into(), dst.1);
        let _ = sock.send_slice(data, ep);
        return true;
    }
    false
}
