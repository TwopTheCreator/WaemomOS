use crate::graphics::{self, Color};
use crate::image::{self, Image};
use crate::ui::icons;

const TITLE_H: usize = 22;

pub fn refresh_desktop() {
    graphics::clear_screen(Color::rgb(32, 34, 36));
    draw_topbar();
    draw_dock();
    // render registered app windows (normal and maximized)
    render_registered_apps();
    // draw mouse cursor last
    crate::mouse::draw_cursor();
}

pub fn draw_topbar() {
    let w = graphics::screen_width();
    graphics::fill_rect(0, 0, w, TITLE_H, Color::TITLE_BG);
    let lang = crate::settings::current().language;
    graphics::draw_text(10, 6, &format!("waemom - {}", lang), Color::TITLE_FG, None);
    // show a hint for console
    graphics::draw_text(140, 6, "Ctrl-Enter: console", Color::TITLE_FG, None);
    // system tray-like utilities (clock, net)
    graphics::draw_text(w.saturating_sub(260), 6, "Wi-Fi: ", Color::TITLE_FG, None);
    if let Some(ssid) = crate::net::wifi::current_ssid() { graphics::draw_text(w.saturating_sub(210), 6, ssid, Color::TITLE_FG, None); }
    let t = crate::pit::format_uptime();
    graphics::draw_text(w.saturating_sub(90), 6, t.as_str(), Color::TITLE_FG, None);
}

pub fn draw_dock() {
    let w = graphics::screen_width();
    let h = graphics::screen_height();
    let dock_h = 72;
    let dock_w = (w as f32 * 0.6) as usize;
    let x = (w - dock_w)/2;
    let y = h.saturating_sub(dock_h + 12);
    // macOS-like rounded translucent dock
    graphics::fill_round_rect(x, y, dock_w, dock_h, 18, Color::TITLE_BG);
    // subtle border
    graphics::draw_rect(x, y, dock_w, dock_h, Color::WIN_BORDER);

    // icons centered
    if !crate::settings::current().icons_enabled { return; }
    let items: [(&str, Option<&Image>); 7] = [
        ("Notes", icons::icon_notes()),
        ("Files", icons::icon_files()),
        ("Settings", icons::icon_settings()),
        ("Tasks", icons::icon_task()),
        ("Broom", icons::icon_broom()),
        ("Network", icons::icon_task()),
        ("About", icons::icon_about()),
    ];
    let count = items.len();
    let spacing = 88usize;
    let total_w = spacing * count;
    let mut cx = x + (dock_w.saturating_sub(total_w))/2 + 16;
    for (name, icon) in items.iter() {
        if let Some(img) = icon { image::draw_image(cx, y + 12, img); }
        let scale = crate::settings::current().font_scale.max(1);
        graphics::draw_text_scaled(cx, y + dock_h - 16, name, Color::WHITE, None, scale);
        cx += spacing;
    }
}

pub fn open_window(x: usize, y: usize, w: usize, h: usize, title: &str, body: &str) {
    open_window_icon(x, y, w, h, title, body, None);
}

pub fn open_window_icon(x: usize, y: usize, w: usize, h: usize, title: &str, body: &str, icon: Option<&Image>) {
    // Window frame
    graphics::fill_rect(x, y, w, h, Color::WIN_BG);
    graphics::draw_rect(x, y, w, h, Color::WIN_BORDER);

    // Title bar + controls
    graphics::fill_rect(x, y, w, TITLE_H, Color::TITLE_BG);
    draw_controls(x + 12, y + (TITLE_H as isize / 2) as usize);
    if crate::settings::current().icons_enabled {
        if let Some(img) = icon { image::draw_image(x + 36, y + 3, img); }
    }
    let scale = crate::settings::current().font_scale.max(1);
    graphics::draw_text_scaled(x + 56, y + 6, title, Color::TITLE_FG, None, scale);

    // Content area
    let pad = 8;
    let tx = x + pad;
    let ty = y + TITLE_H + 6;
    let scale = crate::settings::current().font_scale.max(1);
    graphics::draw_text_scaled(tx, ty, body, Color::WHITE, None, scale);
}

fn draw_controls(cx: usize, cy: usize) {
    let r = 5;
    graphics::fill_circle(cx as isize, cy as isize, r, Color::RED);
    graphics::fill_circle((cx + 16) as isize, cy as isize, r, Color::YELLOW);
    graphics::fill_circle((cx + 32) as isize, cy as isize, r, Color::GREEN);
}

pub fn open_window_animated(x: usize, y: usize, w: usize, h: usize, title: &str, body: &str, steps: usize) {
    open_window_icon_animated(x, y, w, h, title, body, steps, None);
}

pub fn open_window_icon_animated(x: usize, y: usize, w: usize, h: usize, title: &str, body: &str, steps: usize, icon: Option<&Image>) {
    if !crate::settings::current().animations_enabled { refresh_desktop(); open_window_icon(x,y,w,h,title,body,icon); return; }
    for i in 1..=steps {
        let iw = w * i / steps;
        let ih = h * i / steps;
        refresh_desktop();
        open_window_icon(x, y, iw.max(60), ih.max(40), title, body, icon);
        sleep(120_000);
    }
    refresh_desktop();
    open_window_icon(x, y, w, h, title, body, icon);
}

pub fn close_window_animated(x: usize, y: usize, w: usize, h: usize, steps: usize) {
    if !crate::settings::current().animations_enabled { refresh_desktop(); return; }
    for i in (1..=steps).rev() {
        let iw = w * i / steps;
        let ih = h * i / steps;
        refresh_desktop();
        open_window(x, y, iw.max(60), ih.max(40), "", "");
        sleep(120_000);
    }
    refresh_desktop();
}

pub fn maximize_area() -> (usize, usize, usize, usize) {
    let w = graphics::screen_width();
    let h = graphics::screen_height();
    let x = 0; let y = TITLE_H; let ww = w; let hh = h.saturating_sub(TITLE_H + 64);
    (x, y, ww, hh)
}

pub fn sleep(iters: u64) {
    for _ in 0..iters { core::hint::spin_loop(); }
}

fn render_registered_apps() {
    use crate::appmgr::{self, WindowState};
    for app in appmgr::list().iter() {
        match app.state {
            WindowState::Closed | WindowState::Minimized => {}
            WindowState::Normal => open_window_icon(app.x, app.y, app.w, app.h, app.name, &app.body, app.icon),
            WindowState::Maximized => { let (x,y,w,h)=maximize_area(); open_window_icon(x,y,w,h, app.name, &app.body, app.icon) }
        }
    }
}
