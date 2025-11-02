pub mod memfs;
pub mod fat;

use memfs::MemFs;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref FS: Mutex<Option<MemFs>> = Mutex::new(None);
}

pub fn init() {
    let mut fs = MemFs::new_dir("/");
    fs.add_file("/README.txt", b"waemom OS\nThis is a tiny hobby kernel with a toy window system.\n");
    fs.add_file("/etc/motd", b"Welcome to waemom!\nEnjoy your stay.\n");
    // Include a tiny ELF64 header for Linux inspection
    let hello: &'static [u8] = include_bytes!("../../assets/bin/hello");
    fs.add_file("/bin/hello", hello);
    // Include default lock file and web samples
    let lock: &'static [u8] = include_bytes!("../../assets/waemon.lock");
    fs.add_file("/waemon.lock", lock);
    fs.add_file("/www/index.html", include_bytes!("../../assets/www/index.html"));
    fs.add_file("/www/app.js", include_bytes!("../../assets/www/app.js"));
    fs.add_file("/www/styles.css", include_bytes!("../../assets/www/styles.css"));
    *FS.lock() = Some(fs);
}

pub fn read(path: &str) -> Result<Vec<u8>, ()> {
    FS.lock().as_mut().ok_or(())?.read(path)
}

pub fn list(path: &str) -> Result<Vec<String>, ()> {
    FS.lock().as_ref().ok_or(())?.list(path)
}

pub fn write(path: &str, data: &[u8]) -> Result<(), ()> {
    FS.lock().as_mut().ok_or(())?.write(path, data)
}
