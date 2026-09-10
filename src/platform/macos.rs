use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};

use objc2::ffi::{object_getClass, object_setClass};
use objc2::foundation::NSPoint;
use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, Sel};
use objc2::{msg_send, sel};
use objc2_app_kit::{NSWindow, NSWindowSharingType};

static STRIP_POINTS: AtomicI32 = AtomicI32::new(0);
static HIT_TEST_CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

pub fn set_stealth(ns_view_ptr: *mut c_void) {
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

pub fn set_click_through(ns_view_ptr: *mut c_void, enable: bool, topbar_height: f32) {
    if ns_view_ptr.is_null() {
        return;
    }
    STRIP_POINTS.store(
        if enable {
            topbar_height.ceil() as i32
        } else {
            0
        },
        Ordering::Relaxed,
    );
    install_hit_test_subclass(ns_view_ptr);
}

fn install_hit_test_subclass(ns_view_ptr: *mut c_void) {
    if HIT_TEST_CLASS.get().is_some() {
        return;
    }
    unsafe {
        let superclass = object_getClass(ns_view_ptr);
        if superclass.is_null() {
            return;
        }

        let mut builder = ClassBuilder::new(c"SAHitTestView", &*superclass).unwrap_or_else(|| {
            ClassBuilder::new(c"SAHitTestView2", &*superclass)
                .unwrap_or_else(|| ClassBuilder::new(c"SAHitTestView3", &*superclass).unwrap())
        });
        builder.add_method(
            sel!(hitTest:),
            hit_test as extern "C-unwind" fn(&AnyObject, Sel, NSPoint) -> Option<&AnyObject>,
        );
        let class: &'static AnyClass = builder.register();
        let _ = HIT_TEST_CLASS.set(class);

        object_setClass(ns_view_ptr, class as *const AnyClass);
    }
}

/// The winit content view is flipped (origin at the top-left), so the
/// interactive title-bar strip occupies `point.y < STRIP_POINTS`.
unsafe extern "C-unwind" fn hit_test(
    this: &AnyObject,
    _cmd: Sel,
    point: NSPoint,
) -> Option<&AnyObject> {
    let strip = STRIP_POINTS.load(Ordering::Relaxed);
    if strip > 0 && point.y >= strip as f64 {
        return None;
    }

    let Some(&superclass) = HIT_TEST_CLASS.get() else {
        return None;
    };
    let result: Option<&AnyObject> = unsafe { msg_send![super(this, superclass), hitTest: point] };
    result
}
