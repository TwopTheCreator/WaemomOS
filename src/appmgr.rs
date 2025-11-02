use alloc::string::String;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::ui::icons;
use crate::image::Image;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WindowState { Normal, Minimized, Maximized, Closed }

pub struct App {
    pub id: u32,
    pub name: &'static str,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub state: WindowState,
    pub icon: Option<&'static Image>,
    pub body: String,
}

lazy_static! {
    static ref APPS: Mutex<Vec<App>> = Mutex::new(Vec::new());
}

pub fn spawn(name: &'static str, x: usize, y: usize, w: usize, h: usize, icon: Option<&'static Image>, body: String) -> u32 {
    let mut apps = APPS.lock();
    let id = (apps.len() as u32) + 1;
    apps.push(App { id, name, x, y, w, h, state: WindowState::Normal, icon, body });
    id
}

pub fn set_state(id: u32, st: WindowState) {
    let mut apps = APPS.lock();
    if let Some(a) = apps.iter_mut().find(|a| a.id == id) { a.state = st; }
}

pub fn list() -> Vec<App> { APPS.lock().clone() }
