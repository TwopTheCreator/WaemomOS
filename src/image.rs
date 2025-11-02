use crate::graphics::{self, Color};
use alloc::vec::Vec;

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>, // RGB
}

pub fn draw_image(x: usize, y: usize, img: &Image) {
    for j in 0..img.height {
        for i in 0..img.width {
            let idx = (j * img.width + i) * 3;
            let r = img.data[idx]; let g = img.data[idx+1]; let b = img.data[idx+2];
            graphics::put_pixel(x + i, y + j, Color::rgb(r,g,b));
        }
    }
}

// Minimal PPM P3 (ASCII) parser: "P3\n<width> <height>\n255\n" then ASCII triples
pub fn parse_ppm_p3(bytes: &[u8]) -> Option<Image> {
    let s = core::str::from_utf8(bytes).ok()?;
    let mut it = s.split_whitespace();
    if it.next()? != "P3" { return None; }
    let width: usize = it.next()?.parse().ok()?;
    let height: usize = it.next()?.parse().ok()?;
    let maxv: usize = it.next()?.parse().ok()?;
    if maxv == 0 { return None; }
    let mut data = Vec::with_capacity(width*height*3);
    for _ in 0..(width*height) {
        let r: usize = it.next()?.parse().ok()?;
        let g: usize = it.next()?.parse().ok()?;
        let b: usize = it.next()?.parse().ok()?;
        data.push(((r * 255) / maxv) as u8);
        data.push(((g * 255) / maxv) as u8);
        data.push(((b * 255) / maxv) as u8);
    }
    Some(Image { width, height, data })
}
