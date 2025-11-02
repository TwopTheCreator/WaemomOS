use alloc::string::String;
use crate::fs;

pub fn view_html() -> String {
    let mut s = String::from("HTML Viewer\n\n");
    if let Ok(bytes) = fs::read("/www/index.html") {
        s.push_str(core::str::from_utf8(&bytes).unwrap_or("<utf8 error>"));
    } else { s.push_str("/www/index.html not found"); }
    s
}

pub fn view_js() -> String {
    let mut s = String::from("JS Viewer\n\n");
    if let Ok(bytes) = fs::read("/www/app.js") {
        s.push_str(core::str::from_utf8(&bytes).unwrap_or("<utf8 error>"));
    } else { s.push_str("/www/app.js not found"); }
    s
}

pub fn view_css() -> String {
    let mut s = String::from("CSS Viewer\n\n");
    if let Ok(bytes) = fs::read("/www/styles.css") {
        s.push_str(core::str::from_utf8(&bytes).unwrap_or("<utf8 error>"));
    } else { s.push_str("/www/styles.css not found"); }
    s
}
