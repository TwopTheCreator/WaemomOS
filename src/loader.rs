use crate::{graphics, window};
use crate::graphics::Color;
use crate::fs;

pub fn loading_sequence(files: &[&str]) {
    let (w,h) = (graphics::screen_width(), graphics::screen_height());
    let bw = 420usize; let bh = 120usize;
    let x = (w.saturating_sub(bw))/2; let y = (h.saturating_sub(bh))/2;
    let steps = files.len().max(1);
    let mut idx = 0usize;
    for f in files.iter() {
        // backdrop
        graphics::fill_rect(0, 0, w, h, Color::rgb(20,22,24));
        graphics::draw_rect(x, y, bw, bh, Color::WIN_BORDER);
        graphics::fill_rect(x+1, y+1, bw-2, bh-2, Color::WIN_BG);
        let title = "Loading waemom...";
        graphics::draw_text(x+16, y+12, title, Color::WHITE, None);
        let msg = format!("Loading: {}", f);
        graphics::draw_text(x+16, y+36, &msg, Color::WHITE, None);
        // progress bar
        let pw = bw-32; let ph = 14; let px = x+16; let py = y+68;
        graphics::draw_rect(px, py, pw, ph, Color::WIN_BORDER);
        let filled = ((idx+1)* (pw-2)) / steps;
        graphics::fill_rect(px+1, py+1, filled, ph-2, Color::GREEN);
        // spinner
        let frames = ["-","\\","|","/"];
        let fr = frames[idx % frames.len()];
        graphics::draw_text(px+pw-16, py-18, fr, Color::YELLOW, None);
        // simulate IO
        let _ = fs::read(f);
        window::sleep(400_000);
        idx += 1;
    }
}
