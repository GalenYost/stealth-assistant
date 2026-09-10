#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowManager;

#[cfg(target_os = "macos")]
pub mod macos;

use raw_window_handle::RawWindowHandle;

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
