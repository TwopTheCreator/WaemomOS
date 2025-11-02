use lazy_static::lazy_static;
use spin::Mutex;

pub struct Console {
    pub input: heapless::String<128>,
    pub output: heapless::String<512>,
}

lazy_static! {
    static ref CONSOLE: Mutex<Console> = Mutex::new(Console { input: heapless::String::new(), output: heapless::String::new() });
}

pub fn on_char(c: char) {
    let mut cns = CONSOLE.lock();
    match c {
        '\n' | '\r' => {
            handle_command(&mut cns);
        }
        '\u{8}' => { let _ = cns.input.pop(); }
        _ => { let _ = cns.input.push(c); }
    }
    drop(cns);
    draw();
}

fn handle_command(cns: &mut Console) {
    let cmd = cns.input.as_str().trim();
    if !cns.output.is_empty() { let _ = cns.output.push_str("\n"); }
    let _ = cns.output.push_str("> "); let _ = cns.output.push_str(cmd); let _ = cns.output.push_str("\n");
    match cmd {
        "hello" => { let _ = cns.output.push_str("Hello!\n"); }
        "clear" => { cns.output.clear(); }
        _ if cmd.starts_with("sleep ") => {
            if let Ok(t) = cmd[6..].trim().parse::<u64>() { crate::syscalls::sleep_ticks(t); let _ = cns.output.push_str("(sleep requested)\n"); }
        }
        _ if cmd.starts_with("spawn ") => {
            let name = cmd[6..].trim();
            let _pid = crate::syscalls::spawn(name);
            let _ = cns.output.push_str("(spawn requested)\n");
        }
        _ if cmd == "uptime" => {
            let t = crate::pit::format_uptime();
            let _ = cns.output.push_str(t.as_str()); let _ = cns.output.push('\n');
        }
        _ if cmd.is_empty() => {}
        _ => { let _ = cns.output.push_str("Unknown command\n"); }
    }
    cns.input.clear();
}

pub fn draw() {
    use crate::graphics::{self, Color};
    let w = graphics::screen_width();
    let h = graphics::screen_height();
    let bh = 140usize;
    let y = h.saturating_sub(bh);
    graphics::fill_rect(0, y, w, bh, Color::rgb(16,16,16));
    graphics::draw_rect(0, y, w, bh, Color::WIN_BORDER);
    let pad = 8;
    // render output (trim to fit)
    let mut text = heapless::String::<512>::new();
    let out = CONSOLE.lock().output.clone();
    let _ = text.push_str(out.as_str());
    graphics::draw_text(pad, y + pad, text.as_str(), Color::WHITE, None);
    // prompt line reads from TTY buffer length (not shown here), just draw a chevron
    let line = ">";
    graphics::draw_text(pad, y + bh - 24, line, Color::WHITE, None);
}

pub fn println(s: &str) { let mut c = CONSOLE.lock(); let _ = c.output.push_str(s); let _ = c.output.push('\n'); drop(c); draw(); }
pub fn clear() { CONSOLE.lock().output.clear(); draw(); }
