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
}

impl WindowManager {
    pub fn new(handle: RawWindowHandle) -> Self {
        Self {
            handle,
            is_click_through: false,
        }
    }

    pub fn set_click_through(&mut self, click_through: bool) {
        self.is_click_through = click_through;
        apply_click_through(self.handle, click_through);
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

pub fn apply_click_through(handle: RawWindowHandle, enable: bool) {
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(h) = handle {
        windows::set_click_through(h.hwnd.get() as _, enable);
    }

    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(h) = handle {
        macos::set_click_through(h.ns_view.as_ptr(), enable);
    }

    #[cfg(target_os = "linux")]
    if let RawWindowHandle::Xlib(h) = handle {
        linux::set_click_through(Some(h.window as u32), enable);
    }
}
