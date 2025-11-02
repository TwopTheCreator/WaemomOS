use heapless::spsc::Queue;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref TTY0: Mutex<Tty> = Mutex::new(Tty::new());
}

pub struct Tty {
    q: Queue<char, 256>,
}

impl Tty {
    pub const fn new() -> Self { Self { q: Queue::new() } }
}

pub fn write_char(c: char) { let mut t = TTY0.lock(); let _ = t.q.enqueue(c); }

pub fn read_char() -> Option<char> { let mut t = TTY0.lock(); t.q.dequeue() }

pub fn read_line(buf: &mut heapless::String<256>) -> bool {
    while let Some(c) = read_char() {
        match c {
            '\n' | '\r' => return true,
            '\u{8}' => { let _ = buf.pop(); },
            _ => { let _ = buf.push(c); },
        }
    }
    false
}
