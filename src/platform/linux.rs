use x11rb::connection::Connection;
use x11rb::protocol::shape::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{self, AtomEnum, ConnectionExt as _, PropMode, Window};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

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

pub fn set_click_through(
    x11_window: Option<u32>,
    enable: bool,
    topbar_height: f32,
    pixels_per_point: f32,
) {
    let window_id = match x11_window {
        Some(id) => id as Window,
        None => return,
    };

    if let Ok((conn, _)) = RustConnection::connect(None) {
        if enable {
            let strip_height_px = (topbar_height * pixels_per_point).round().max(1.0) as u16;
            let _ = conn.shape_rectangles(
                shape::SO::SET,
                shape::SK::INPUT,
                xproto::ClipOrdering::UNSORTED,
                window_id,
                0,
                0,
                &[xproto::Rectangle {
                    x: 0,
                    y: 0,
                    width: 40000,
                    height: strip_height_px,
                }],
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
