use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GWLP_WNDPROC, GetWindowLongPtrW, GetWindowRect, HTCLIENT, HTTRANSPARENT,
    SetWindowDisplayAffinity, SetWindowLongPtrW, WDA_EXCLUDEFROMCAPTURE, WM_NCHITTEST,
};

static STRIP_PIXELS: AtomicIsize = AtomicIsize::new(0);
static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);

pub fn set_stealth(hwnd_ptr: isize) {
    if hwnd_ptr == 0 {
        return;
    }
    unsafe {
        let hwnd = HWND(hwnd_ptr as *mut _);
        let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
    }
}

pub fn set_click_through(hwnd_ptr: isize, enable: bool, topbar_height: f32, pixels_per_point: f32) {
    if hwnd_ptr == 0 {
        return;
    }

    let strip_pixels = if enable {
        (topbar_height * pixels_per_point).round().max(1.0) as isize
    } else {
        0
    };
    STRIP_PIXELS.store(strip_pixels, Ordering::Relaxed);

    let subclass =
        subclass_wnd_proc as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;
    let subclass_ptr = subclass as *const () as isize;

    unsafe {
        let hwnd = HWND(hwnd_ptr as *mut _);
        let current = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
        if current == 0 {
            return;
        }
        if current != subclass_ptr {
            ORIGINAL_WNDPROC.store(current, Ordering::Relaxed);
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, subclass_ptr);
        }
    }
}

unsafe extern "system" fn subclass_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCHITTEST {
        let strip_pixels = STRIP_PIXELS.load(Ordering::Relaxed);
        if strip_pixels > 0 {
            let mut rect = RECT::default();
            if unsafe { GetWindowRect(hwnd, &mut rect).is_ok() } {
                let cursor_y = ((lparam.0 as usize >> 16) & 0xFFFF) as isize;
                if cursor_y > rect.top as isize + strip_pixels {
                    return LRESULT(HTTRANSPARENT as isize);
                }
                return LRESULT(HTCLIENT as isize);
            }
        }
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
