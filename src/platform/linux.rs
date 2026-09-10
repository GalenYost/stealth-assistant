use raw_window_handle::{RawWindowHandle, XcbWindowHandle, XlibWindowHandle};
use x11rb::connection::Connection;
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{self, AtomEnum, ConnectionExt as _, PropMode, Window};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

pub struct WindowManager {
    pub window_id: Option<u32>,
    pub is_click_through: bool,
}

impl WindowManager {
    pub fn new(window_id: Option<u32>) -> Self {
        Self {
            window_id,
            is_click_through: false,
        }
    }

    pub fn set_click_through(&mut self, click_through: bool) {
        self.is_click_through = click_through;
        set_click_through(self.window_id, click_through);
    }
}

pub fn extract_x11_window_id(handle: RawWindowHandle) -> Option<u32> {
    match handle {
        RawWindowHandle::Xlib(XlibWindowHandle { window, .. }) => Some(window as u32),
        RawWindowHandle::Xcb(XcbWindowHandle { window, .. }) => Some(window.get()),
        _ => None,
    }
}

pub fn apply_stealth(handle: RawWindowHandle) {
    if let Some(win_id) = extract_x11_window_id(handle) {
        set_stealth(Some(win_id));
    }
}

pub fn set_stealth(x11_window: Option<u32>) {
    let window_id = match x11_window {
        Some(id) => id as Window,
        None => return,
    };

    if let Ok((conn, _)) = RustConnection::connect(None) {
        if let (Ok(atom_type), Ok(atom_dock)) = (
            conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE"),
            conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_NOTIFICATION"),
        ) {
            if let (Ok(type_reply), Ok(dock_reply)) = (atom_type.reply(), atom_dock.reply()) {
                let _ = conn.change_property32(
                    PropMode::REPLACE,
                    window_id,
                    type_reply.atom,
                    AtomEnum::ATOM,
                    &[dock_reply.atom],
                );
                let _ = conn.flush();
            }
        }
    }
}

pub fn set_click_through(x11_window: Option<u32>, enable: bool) {
    let window_id = match x11_window {
        Some(id) => id as Window,
        None => return,
    };

    if let Ok((conn, _)) = RustConnection::connect(None) {
        if enable {
            let _ = conn.shape_rectangles(
                shape::SO::SET,
                shape::SK::INPUT,
                xproto::ClipOrdering::UNSORTED,
                window_id,
                0,
                0,
                &[],
            );
        } else {
            let _ = conn.shape_mask(
                shape::SO::SET,
                shape::SK::INPUT,
                window_id,
                0,
                0,
                x11rb::NONE,
            );
        }
        let _ = conn.flush();
    }
}
