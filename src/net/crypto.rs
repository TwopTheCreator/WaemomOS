// Demo-only XOR cipher to “encrypt” IP payloads; not secure.
pub fn encrypt_in_place(buf: &mut [u8], key: u8) { for b in buf { *b ^= key; } }
pub fn decrypt_in_place(buf: &mut [u8], key: u8) { encrypt_in_place(buf, key) }
