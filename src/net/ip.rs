pub fn dhcp() {
    // Simulate DHCP by assigning a typical QEMU NAT IP
    super::set_ip([10,0,2,15]);
}
