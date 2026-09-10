use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowDisplayAffinity, SetWindowLongPtrW, GWL_EXSTYLE,
    WDA_EXCLUDEFROMCAPTURE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};

pub fn set_stealth(hwnd_ptr: isize) {
    if hwnd_ptr == 0 {
        return;
    }
    unsafe {
        let hwnd = HWND(hwnd_ptr as *mut _);
        let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
    }
}

pub fn set_click_through(hwnd_ptr: isize, enable: bool) {
    if hwnd_ptr == 0 {
        return;
    }
    unsafe {
        let hwnd = HWND(hwnd_ptr as *mut _);
        let current_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;

        let new_style = if enable {
            current_style | WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0
        } else {
            current_style & !WS_EX_TRANSPARENT.0
        };

        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style as isize);
    }
}
