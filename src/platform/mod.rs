#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use raw_window_handle::RawWindowHandle;

pub struct WindowManager {
    pub handle: RawWindowHandle,
    pub is_click_through: bool,
    topbar_points: f32,
    pixels_per_point: f32,
}

impl WindowManager {
    pub fn new(handle: RawWindowHandle) -> Self {
        Self {
            handle,
            is_click_through: false,
            topbar_points: 0.0,
            pixels_per_point: 1.0,
        }
    }

    /// Sets the interactive title-bar strip height (in egui points).
    /// Re-applies the click-through region when it changes.
    pub fn set_topbar_height(&mut self, points: f32, pixels_per_point: f32) {
        self.topbar_points = points;
        self.pixels_per_point = pixels_per_point;
        self.apply_click_through();
    }

    pub fn set_click_through(&mut self, click_through: bool) {
        self.is_click_through = click_through;
        self.apply_click_through();
    }

    fn apply_click_through(&self) {
        apply_click_through(
            self.handle,
            self.is_click_through,
            self.topbar_points,
            self.pixels_per_point,
        );
    }
}

pub fn apply_stealth(handle: RawWindowHandle) {
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(h) = handle {
        windows::set_stealth(h.hwnd.get() as _);
    }

    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(h) = handle {
        macos::set_stealth(h.ns_view.as_ptr());
    }

    #[cfg(target_os = "linux")]
    if let RawWindowHandle::Xlib(h) = handle {
        linux::set_stealth(Some(h.window as u32));
    }
}

/// Enables or disables click-through on everything EXCEPT the title bar strip.
/// `topbar_height` is in egui points; `pixels_per_point` is the current scale
/// factor (ignored on macOS, where hit testing happens in points).
pub fn apply_click_through(
    handle: RawWindowHandle,
    enable: bool,
    topbar_height: f32,
    pixels_per_point: f32,
) {
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(h) = handle {
        windows::set_click_through(h.hwnd.get() as _, enable, topbar_height, pixels_per_point);
    }

    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(h) = handle {
        macos::set_click_through(h.ns_view.as_ptr(), enable, topbar_height);
    }

    #[cfg(target_os = "linux")]
    if let RawWindowHandle::Xlib(h) = handle {
        linux::set_click_through(
            Some(h.window as u32),
            enable,
            topbar_height,
            pixels_per_point,
        );
    }
}
