use objc2_app_kit::{NSWindow, NSWindowSharingType};

pub fn set_stealth(ns_view_ptr: *mut std::ffi::c_void) {
    if ns_view_ptr.is_null() {
        return;
    }
    unsafe {
        let view: *mut objc2::ffi::objc_object = ns_view_ptr.cast();
        let window_sel = objc2::sel!(window);
        let window: *mut NSWindow = objc2::msg_send![view, performSelector: window_sel];

        if !window.is_null() {
            let _: () = objc2::msg_send![
                window,
                setSharingType: NSWindowSharingType::None
            ];
        }
    }
}

pub fn set_click_through(ns_view_ptr: *mut std::ffi::c_void, enable: bool) {
    if ns_view_ptr.is_null() {
        return;
    }
    unsafe {
        let view: *mut objc2::ffi::objc_object = ns_view_ptr.cast();
        let window_sel = objc2::sel!(window);
        let window: *mut NSWindow = objc2::msg_send![view, performSelector: window_sel];

        if !window.is_null() {
            let _: () = objc2::msg_send![window, setIgnoresMouseEvents: enable];
        }
    }
}
