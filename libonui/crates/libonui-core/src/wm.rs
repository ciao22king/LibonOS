//! The window manager and the desktop session.
//!
//! This is deliberately toolkit-free: it owns the geometry, stacking and focus
//! rules, and the UI layer only draws what it is told. Keeping it here means the
//! behaviour can be tested without a display.

use crate::system::{pinned_apps, PreinstalledApp};

/// Identifies one open window for the lifetime of the session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(pub u64);

/// How a window is currently presented.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Maximized,
    Minimized,
}

/// A single application window.
#[derive(Clone, Debug)]
pub struct Window {
    pub id: WindowId,
    pub app_id: String,
    pub title: String,
    /// Restored geometry, used when un-maximising.
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub state: WindowState,
    pub z: u32,
    pub resizable: bool,
}

impl Window {
    pub fn is_visible(&self) -> bool {
        self.state != WindowState::Minimized
    }
}

/// The stacking and focus manager.
#[derive(Clone, Debug)]
pub struct WindowManager {
    windows: Vec<Window>,
    focused: Option<WindowId>,
    next_id: u64,
    next_z: u32,
    /// The desktop area windows may occupy, excluding the panel and dock.
    pub viewport_w: f32,
    pub viewport_h: f32,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new(1440.0, 900.0)
    }
}

impl WindowManager {
    pub fn new(viewport_w: f32, viewport_h: f32) -> Self {
        Self {
            windows: Vec::new(),
            focused: None,
            next_id: 1,
            next_z: 1,
            viewport_w,
            viewport_h,
        }
    }

    /// Open a window, cascading it so windows never land exactly on top of each
    /// other, and give it focus.
    pub fn open(&mut self, app_id: impl Into<String>, title: impl Into<String>) -> WindowId {
        let id = WindowId(self.next_id);
        self.next_id += 1;

        let (w, h) = (self.default_width(), self.default_height());
        let step = 28.0;
        let cascade = (self.windows.len() % 6) as f32 * step;
        let x = (120.0 + cascade).min((self.viewport_w - w - 16.0).max(16.0));
        let y = (72.0 + cascade).min((self.viewport_h - h - 16.0).max(16.0));

        let z = self.next_z;
        self.next_z += 1;

        self.windows.push(Window {
            id,
            app_id: app_id.into(),
            title: title.into(),
            x,
            y,
            w,
            h,
            state: WindowState::Normal,
            z,
            resizable: true,
        });
        self.focused = Some(id);
        id
    }

    fn default_width(&self) -> f32 {
        (self.viewport_w * 0.62).clamp(520.0, 1040.0)
    }

    fn default_height(&self) -> f32 {
        (self.viewport_h * 0.68).clamp(380.0, 720.0)
    }

    pub fn close(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
        if self.focused == Some(id) {
            self.focused = self.topmost().map(|w| w.id);
        }
    }

    /// Raise a window and focus it.
    pub fn focus(&mut self, id: WindowId) {
        if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
            if win.state == WindowState::Minimized {
                win.state = WindowState::Normal;
            }
            win.z = self.next_z;
            self.next_z += 1;
            self.focused = Some(id);
        }
    }

    pub fn minimize(&mut self, id: WindowId) {
        if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
            win.state = WindowState::Minimized;
        }
        if self.focused == Some(id) {
            self.focused = self.topmost().map(|w| w.id);
        }
    }

    pub fn toggle_maximize(&mut self, id: WindowId) {
        if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
            win.state = match win.state {
                WindowState::Maximized => WindowState::Normal,
                _ => WindowState::Maximized,
            };
            win.z = self.next_z;
            self.next_z += 1;
        }
        self.focused = Some(id);
    }

    /// Move a normal window, keeping its title bar on screen.
    pub fn move_by(&mut self, id: WindowId, dx: f32, dy: f32) {
        if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
            if win.state != WindowState::Normal {
                return;
            }
            win.x = (win.x + dx).clamp(-win.w + 120.0, self.viewport_w - 120.0);
            win.y = (win.y + dy).clamp(0.0, self.viewport_h - 44.0);
        }
    }

    pub fn resize_by(&mut self, id: WindowId, dw: f32, dh: f32) {
        if let Some(win) = self.windows.iter_mut().find(|w| w.id == id) {
            if win.state != WindowState::Normal || !win.resizable {
                return;
            }
            win.w = (win.w + dw).clamp(360.0, self.viewport_w);
            win.h = (win.h + dh).clamp(240.0, self.viewport_h);
        }
    }

    pub fn set_viewport(&mut self, w: f32, h: f32) {
        self.viewport_w = w;
        self.viewport_h = h;
    }

    pub fn get(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.windows.len()
    }

    pub fn focused_id(&self) -> Option<WindowId> {
        self.focused
    }

    pub fn focused(&self) -> Option<&Window> {
        self.focused.and_then(|id| self.get(id))
    }

    pub fn topmost(&self) -> Option<&Window> {
        self.windows
            .iter()
            .filter(|w| w.is_visible())
            .max_by_key(|w| w.z)
    }

    /// Visible windows from the back to the front — the order to draw them in.
    pub fn visible_back_to_front(&self) -> Vec<&Window> {
        let mut v: Vec<&Window> = self.windows.iter().filter(|w| w.is_visible()).collect();
        v.sort_by_key(|w| w.z);
        v
    }

    /// Whether an app has at least one window open (the dock shows a dot).
    pub fn app_is_running(&self, app_id: &str) -> bool {
        self.windows.iter().any(|w| w.app_id == app_id)
    }

    /// Toggle an app: focus its window if open, otherwise the caller opens one.
    pub fn existing_window_for(&self, app_id: &str) -> Option<WindowId> {
        self.windows
            .iter()
            .filter(|w| w.app_id == app_id)
            .max_by_key(|w| w.z)
            .map(|w| w.id)
    }
}

/// One entry in the dock.
#[derive(Clone, Debug)]
pub struct DockItem {
    pub app_id: String,
    pub name: String,
    pub pinned: bool,
}

/// The running session: who is logged in, the window manager, and the dock.
#[derive(Clone, Debug)]
pub struct Session {
    pub user_first_name: String,
    pub username: String,
    pub wm: WindowManager,
    pub dock: Vec<DockItem>,
    pub start_menu_open: bool,
}

impl Session {
    pub fn new(user_first_name: impl Into<String>, username: impl Into<String>) -> Self {
        let dock = pinned_apps()
            .into_iter()
            .map(|a: &PreinstalledApp| DockItem {
                app_id: a.id.to_string(),
                name: a.name.to_string(),
                pinned: true,
            })
            .collect();
        Self {
            user_first_name: user_first_name.into(),
            username: username.into(),
            wm: WindowManager::default(),
            dock,
            start_menu_open: false,
        }
    }

    /// Launch an app: focus it if already open, otherwise open a window.
    pub fn launch(&mut self, app_id: &str, title: &str) {
        self.start_menu_open = false;
        if let Some(id) = self.wm.existing_window_for(app_id) {
            self.wm.focus(id);
        } else {
            self.wm.open(app_id, title);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_focuses_and_cascades() {
        let mut wm = WindowManager::new(1440.0, 900.0);
        let a = wm.open("files", "File");
        let b = wm.open("terminal", "Terminale");
        assert_eq!(wm.focused_id(), Some(b));
        assert_ne!(wm.get(a).unwrap().x, wm.get(b).unwrap().x);
        assert_eq!(wm.len(), 2);
    }

    #[test]
    fn focus_raises_to_front() {
        let mut wm = WindowManager::new(1440.0, 900.0);
        let a = wm.open("files", "File");
        let b = wm.open("terminal", "Terminale");
        wm.focus(a);
        assert_eq!(wm.topmost().unwrap().id, a);
        assert_eq!(wm.focused_id(), Some(a));
        assert!(wm.get(a).unwrap().z > wm.get(b).unwrap().z);
    }

    #[test]
    fn minimize_hides_and_refocuses() {
        let mut wm = WindowManager::new(1440.0, 900.0);
        let a = wm.open("files", "File");
        let b = wm.open("terminal", "Terminale");
        wm.minimize(b);
        assert!(!wm.get(b).unwrap().is_visible());
        assert_eq!(wm.focused_id(), Some(a));
        assert_eq!(wm.visible_back_to_front().len(), 1);
        wm.focus(b);
        assert!(wm.get(b).unwrap().is_visible());
    }

    #[test]
    fn maximize_toggles_and_blocks_moving() {
        let mut wm = WindowManager::new(1440.0, 900.0);
        let a = wm.open("files", "File");
        let before = wm.get(a).unwrap().x;
        wm.toggle_maximize(a);
        assert_eq!(wm.get(a).unwrap().state, WindowState::Maximized);
        wm.move_by(a, 50.0, 50.0);
        assert_eq!(
            wm.get(a).unwrap().x,
            before,
            "maximized windows do not move"
        );
        wm.toggle_maximize(a);
        assert_eq!(wm.get(a).unwrap().state, WindowState::Normal);
    }

    #[test]
    fn resize_respects_minimums() {
        let mut wm = WindowManager::new(1440.0, 900.0);
        let a = wm.open("files", "File");
        wm.resize_by(a, -10_000.0, -10_000.0);
        assert_eq!(wm.get(a).unwrap().w, 360.0);
        assert_eq!(wm.get(a).unwrap().h, 240.0);
    }

    #[test]
    fn closing_falls_back_to_the_next_window() {
        let mut wm = WindowManager::new(1440.0, 900.0);
        let a = wm.open("files", "File");
        let b = wm.open("terminal", "Terminale");
        wm.close(b);
        assert_eq!(wm.focused_id(), Some(a));
        wm.close(a);
        assert!(wm.is_empty());
        assert_eq!(wm.focused_id(), None);
    }

    #[test]
    fn launching_an_open_app_focuses_instead_of_duplicating() {
        let mut s = Session::new("Giulia", "giulia");
        s.launch("files", "File");
        s.launch("files", "File");
        assert_eq!(s.wm.len(), 1);
        assert!(s.wm.app_is_running("files"));
    }

    #[test]
    fn dock_starts_with_the_pinned_apps() {
        let s = Session::new("Giulia", "giulia");
        let ids: Vec<_> = s.dock.iter().map(|d| d.app_id.as_str()).collect();
        assert_eq!(ids, ["firefox", "files", "terminal", "store", "settings"]);
        assert!(s.dock.iter().all(|d| d.pinned));
    }
}
