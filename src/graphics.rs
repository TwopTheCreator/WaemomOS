use bootloader_api::info::FrameBuffer;
use core::ops::{Deref, DerefMut};
use lazy_static::lazy_static;
use spin::Mutex;

pub struct Fb {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bpp: usize,
    pub buf: &'static mut [u8],
}

impl Fb {
    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize { y * self.stride + x * (self.bpp / 8) }
}

pub struct Color(pub u8, pub u8, pub u8);
impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self { Self(r, g, b) }
    pub const BLACK: Self = Self(0,0,0);
    pub const WHITE: Self = Self(255,255,255);
    pub const TITLE_BG: Self = Self(50,52,56);
    pub const TITLE_FG: Self = Self(230,230,230);
    pub const WIN_BG: Self = Self(36,38,40);
    pub const WIN_BORDER: Self = Self(85,85,90);
    pub const RED: Self = Self(232, 84, 68);
    pub const YELLOW: Self = Self(240, 194, 67);
    pub const GREEN: Self = Self(77, 198, 94);
}

lazy_static! {
    static ref FB: Mutex<Option<Fb>> = Mutex::new(None);
}

pub fn init(boot_info: &'static bootloader_api::BootInfo) -> Option<()> {
    let fb_info = boot_info.framebuffer.as_ref()?;
    let mut fb = fb_info.buffer_mut();
    let info = fb_info.info();
    let new_fb = Fb {
        width: info.width,
        height: info.height,
        stride: info.stride,
        bpp: info.bytes_per_pixel * 8,
        buf: fb.deref_mut(),
    };
    *FB.lock() = Some(new_fb);
    Some(())
}

fn with_fb<R>(f: impl FnOnce(&mut Fb) -> R) -> Option<R> {
    let mut guard = FB.lock();
    let fb = guard.as_mut()?;
    Some(f(fb))
}

pub fn clear_screen(color: Color) { fill_rect(0, 0, screen_width(), screen_height(), color); }

pub fn screen_width() -> usize { FB.lock().as_ref().map(|f| f.width).unwrap_or(0) }
pub fn screen_height() -> usize { FB.lock().as_ref().map(|f| f.height).unwrap_or(0) }

pub fn put_pixel(x: usize, y: usize, c: Color) {
    let _ = with_fb(|fb| {
        if x >= fb.width || y >= fb.height { return; }
        let i = fb.idx(x, y);
        if fb.bpp == 32 {
            fb.buf[i..i+4].copy_from_slice(&[c.2, c.1, c.0, 0xFF]);
        } else if fb.bpp == 24 {
            fb.buf[i..i+3].copy_from_slice(&[c.2, c.1, c.0]);
        }
    });
}

pub fn fill_rect(x: usize, y: usize, w: usize, h: usize, c: Color) {
    let _ = with_fb(|fb| {
        let max_x = (x + w).min(fb.width);
        let max_y = (y + h).min(fb.height);
        for yy in y..max_y {
            let mut i = fb.idx(x, yy);
            for _xx in x..max_x {
                if fb.bpp == 32 {
                    fb.buf[i..i+4].copy_from_slice(&[c.2, c.1, c.0, 0xFF]);
                    i += 4;
                } else {
                    fb.buf[i..i+3].copy_from_slice(&[c.2, c.1, c.0]);
                    i += 3;
                }
            }
        }
    });
}

pub fn fill_circle(cx: isize, cy: isize, r: isize, c: Color) {
    let rsq = r * r;
    for dy in -r..=r {
        let y = cy + dy;
        for dx in -r..=r {
            if dx*dx + dy*dy <= rsq {
                let x = cx + dx;
                if x >= 0 && y >= 0 { put_pixel(x as usize, y as usize, c); }
            }
        }
    }
}

pub fn draw_rect(x: usize, y: usize, w: usize, h: usize, c: Color) {
    for xx in x..(x+w) { put_pixel(xx, y, c); }
    for xx in x..(x+w) { put_pixel(xx, y+h.saturating_sub(1), c); }
    for yy in y..(y+h) { put_pixel(x, yy, c); }
    for yy in y..(y+h) { put_pixel(x+w.saturating_sub(1), yy, c); }
}

pub fn fill_round_rect(x: usize, y: usize, w: usize, h: usize, r: usize, c: Color) {
    // center
    if w > 2*r { fill_rect(x + r, y, w - 2*r, h, c); }
    // left and right rectangles
    if r > 0 { fill_rect(x, y + r, r, h.saturating_sub(2*r), c); }
    if r > 0 { fill_rect(x + w.saturating_sub(r), y + r, r, h.saturating_sub(2*r), c); }
    // corners
    let r_i = r as isize;
    for dy in -(r_i)..=r_i {
        for dx in -(r_i)..=r_i {
            if dx*dx + dy*dy <= (r_i*r_i) {
                // top-left
                let px = x as isize + r_i + dx;
                let py = y as isize + r_i + dy;
                if px>=0 && py>=0 { put_pixel(px as usize, py as usize, c); }
                // top-right
                let px2 = (x + w - r) as isize + dx;
                let py2 = (y + r) as isize + dy;
                if px2>=0 && py2>=0 { put_pixel(px2 as usize, py2 as usize, c); }
                // bottom-left
                let px3 = (x + r) as isize + dx;
                let py3 = (y + h - r) as isize + dy;
                if px3>=0 && py3>=0 { put_pixel(px3 as usize, py3 as usize, c); }
                // bottom-right
                let px4 = (x + w - r) as isize + dx;
                let py4 = (y + h - r) as isize + dy;
                if px4>=0 && py4>=0 { put_pixel(px4 as usize, py4 as usize, c); }
            }
        }
    }
}

mod font;
use font::{FONT8X8_BASIC, FONT_W, FONT_H};

pub fn draw_char(x: usize, y: usize, ch: char, fg: Color, bg: Option<Color>) {
    let idx = ch as usize;
    if idx >= FONT8X8_BASIC.len() { return; }
    let glyph = &FONT8X8_BASIC[idx];
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8 {
            let on = (bits >> col) & 1 == 1;
            if on { put_pixel(x + col, y + row, fg); } else if let Some(cbg) = bg { put_pixel(x + col, y + row, cbg); }
        }
    }
}

pub fn draw_char_scaled(x: usize, y: usize, ch: char, fg: Color, bg: Option<Color>, scale: usize) {
    if scale <= 1 { return draw_char(x, y, ch, fg, bg); }
    let idx = ch as usize;
    if idx >= FONT8X8_BASIC.len() { return; }
    let glyph = &FONT8X8_BASIC[idx];
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8 {
            let on = (bits >> col) & 1 == 1;
            let px = x + col*scale;
            let py = y + row*scale;
            if on { fill_rect(px, py, scale, scale, fg); } else if let Some(cbg) = bg { fill_rect(px, py, scale, scale, cbg); }
        }
    }
}

pub fn draw_text(mut x: usize, mut y: usize, s: &str, fg: Color, bg: Option<Color>) {
    for ch in s.chars() {
        match ch {
            '\n' => { y += FONT_H; x = 0; },
            _ => { draw_char(x, y, ch, fg, bg); x += FONT_W; }
        }
    }
}

pub fn draw_text_scaled(mut x: usize, mut y: usize, s: &str, fg: Color, bg: Option<Color>, scale: usize) {
    if scale <= 1 { return draw_text(x, y, s, fg, bg); }
    for ch in s.chars() {
        match ch {
            '\n' => { y += FONT_H*scale; x = 0; },
            _ => { draw_char_scaled(x, y, ch, fg, bg, scale); x += FONT_W*scale; }
        }
    }
}
