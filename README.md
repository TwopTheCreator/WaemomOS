# waemom

A tiny hobby operating system kernel that boots on x86_64 and prints to the VGA text buffer.

## Quick start

1. Install Rust nightly and tools:
   - `rustup toolchain install nightly`
   - `rustup component add rust-src llvm-tools-preview --toolchain nightly`
   - `cargo install bootimage`
   - `rustup target add x86_64-unknown-none`

2. Build a bootable image:
   - `cargo +nightly bootimage`

3. Run (requires QEMU):
   - `qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-waemom.bin`

You should see "waemom kernel booting..." on the screen. Serial logs go to COM1.

## Project layout
- `src/main.rs`: kernel entry and panic handler
- `src/vga_buffer.rs`: VGA text mode writer and `println!`
- `src/serial.rs`: serial logger and `serial_println!`

## Next steps
- Interrupt descriptor table (IDT) and timer interrupts
- Memory management: paging, allocator
- Drivers (keyboard, disk)
- Syscalls and userland
