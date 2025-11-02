pub fn inspect_elf64(bytes: &[u8]) -> alloc::string::String {
    use alloc::string::String;
    let mut out = String::new();
    if bytes.len() < 64 { out.push_str("Not ELF64: too small\n"); return out; }
    if &bytes[0..4] != b"\x7FELF" { out.push_str("Not ELF magic\n"); return out; }
    let class = bytes[4]; // 1=32,2=64
    let osabi = bytes[7];
    if class != 2 { out.push_str("Not 64-bit ELF\n"); return out; }
    out.push_str("ELF64 detected\n");
    match osabi {
        0 => out.push_str("OSABI: System V (often Linux)\n"),
        3 => out.push_str("OSABI: Linux\n"),
        v => {
            use core::fmt::Write as _;
            let _ = write!(&mut out, "OSABI: other 0x{:02x}\n", v);
        }
    }
    let e_type = u16::from_le_bytes([bytes[16], bytes[17]]);
    let e_machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    let e_entry = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    let phoff = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
    let phentsize = u16::from_le_bytes(bytes[54..56].try_into().unwrap()) as u64;
    let phnum = u16::from_le_bytes(bytes[56..58].try_into().unwrap()) as u64;
    out.push_str(&format!("Type: 0x{:x}, Machine: 0x{:x}\n", e_type, e_machine));
    out.push_str(&format!("Entry: 0x{:016x}\n", e_entry));
    out.push_str(&format!("PH num: {} size: {} off: {}\n", phnum, phentsize, phoff));
    out
}

fn hex2(v: u8) -> &'static str { const H: [&str; 256] = hex_table(); H[v as usize] }
const fn hex_table() -> [&'static str; 256] {
    let mut t: [&str; 256] = ["00"; 256];
    let mut i = 0;
    while i < 256 { t[i] = HEX[i]; i += 1; }
    t
}
static HEX: [&str; 256] = {
    const DIG: &[u8; 16] = b"0123456789abcdef";
    let mut arr: [&str; 256] = ["00"; 256];
    let mut i = 0;
    while i < 256 {
        let hi = (i >> 4) & 0xF; let lo = i & 0xF;
        arr[i] = STRS[(hi<<4)|lo];
        i += 1;
    }
    arr
};
static STRS: [&str; 256] = gen_hex();
const fn gen_hex() -> [&'static str; 256] {
    const DIG: &[u8; 16] = b"0123456789abcdef";
    let mut out: [&str; 256] = ["00"; 256];
    let mut i = 0;
    while i < 256 {
        let hi = (i >> 4) & 0xF; let lo = i & 0xF;
        out[i] = match (DIG[hi] as char, DIG[lo] as char) {
            ('0','0')=>'00',('0','1')=>'01',('0','2')=>'02',('0','3')=>'03',('0','4')=>'04',('0','5')=>'05',('0','6')=>'06',('0','7')=>'07',('0','8')=>'08',('0','9')=>'09',
            ('0','a')=>'0a',('0','b')=>'0b',('0','c')=>'0c',('0','d')=>'0d',('0','e')=>'0e',('0','f')=>'0f',
            _=>"ff"
        };
        i+=1;
    }
    out
}
