use alloc::string::String;
use alloc::vec::Vec;
use crate::{fs, window, ui::icons};

pub struct Broom {
    pub current_url: String,
    pub history: Vec<String>,
    pub bookmarks: Vec<String>,
}

impl Broom {
    pub fn load() -> Self {
        let history = read_lines("/home/broom/history.txt");
        let bookmarks = read_lines("/home/broom/bookmarks.txt");
        Self { current_url: String::from("about:blank"), history, bookmarks }
    }

    pub fn save(&self) {
        let _ = fs::write("/home/broom/history.txt", self.history.join("\n").as_bytes());
        let _ = fs::write("/home/broom/bookmarks.txt", self.bookmarks.join("\n").as_bytes());
    }

    pub fn search_google(&mut self, query: &str) {
        let enc = url_encode(query);
        self.current_url = format!("https://www.google.com/search?q={}", enc);
        self.history.push(self.current_url.clone());
        self.save();
    }

    pub fn open(&mut self, url: &str) {
        self.current_url = url.to_string();
        self.history.push(self.current_url.clone());
        self.save();
    }

    pub fn view(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Broom\nURL: {}\n\n", self.current_url));
        if self.current_url.starts_with("file://") {
            if let Some(path) = self.current_url.strip_prefix("file://") {
                if let Ok(bytes) = fs::read(path) {
                    s.push_str(core::str::from_utf8(&bytes).unwrap_or("<binary>"));
                    return s;
                }
            }
            s.push_str("File not found\n");
        } else if self.current_url.starts_with("about:") {
            s.push_str("about:blank\nType a query in code to search.\n");
        } else if self.current_url.starts_with("https://www.google.com/search?q=") {
            s.push_str("Google search URL generated.\nNetworking not available in this demo kernel.\n");
        } else {
            s.push_str("Unsupported scheme.\n");
        }
        s
    }
}

fn read_lines(path: &str) -> Vec<String> {
    fs::read(path).ok()
        .and_then(|b| core::str::from_utf8(&b).ok().map(|t| t.lines().map(|l| l.to_string()).collect()))
        .unwrap_or_else(|| Vec::new())
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match b {
            b'0'..=b'9'|b'A'..=b'Z'|b'a'..=b'z'|b'-'|b'_'|b'.'|b'~' => out.push(*b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn launch_demo() {
    let mut br = Broom::load();
    br.search_google("waemom os");
    let view = br.view();
    window::open_window_icon_animated(980, 60, 560, 300, "Broom", &view, 16, icons::icon_files());
}
