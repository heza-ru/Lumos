use cocoa::appkit::{
    NSBackingStoreType, NSColor, NSScreen, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::NSRect;
use objc::{class, msg_send, sel, sel_impl};

/// Window level that floats above fullscreen apps.
/// kCGScreenSaverWindowLevel = 1000; +1 guarantees topmost position.
const OVERLAY_LEVEL: i64 = 1001;

/// Create a transparent, borderless, non-activating NSPanel spanning the
/// primary display. Returns the raw NSPanel pointer.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn create_overlay() -> *mut objc::runtime::Object {
    let screen: id = msg_send![class!(NSScreen), mainScreen];
    let frame: NSRect = msg_send![screen, frame];

    // NSBorderlessWindowMask | NSNonactivatingPanelMask
    let style: u64 = 0 | (1 << 7); // NSNonactivatingPanelMask = 128

    let panel: id = {
        let alloc: id = msg_send![class!(NSPanel), alloc];
        msg_send![alloc,
            initWithContentRect: frame
            styleMask: style
            backing: NSBackingStoreType::NSBackingStoreBuffered
            defer: NO
        ]
    };

    // Transparent background
    let clear: id = msg_send![class!(NSColor), clearColor];
    let _: () = msg_send![panel, setBackgroundColor: clear];
    let _: () = msg_send![panel, setOpaque: NO];
    let _: () = msg_send![panel, setHasShadow: NO];

    // Float above fullscreen apps
    let _: () = msg_send![panel, setLevel: OVERLAY_LEVEL];

    // Appear on ALL Spaces AND over fullscreen apps
    // NSWindowCollectionBehaviorCanJoinAllSpaces    = 1 << 0 =   1
    // NSWindowCollectionBehaviorStationary          = 1 << 4 =  16
    // NSWindowCollectionBehaviorIgnoresCycle        = 1 << 6 =  64
    // NSWindowCollectionBehaviorFullScreenAuxiliary = 1 << 8 = 256
    let behaviors: u64 = 1 | 16 | 64 | 256;
    let _: () = msg_send![panel, setCollectionBehavior: behaviors];

    // Start click-through (pointer mode)
    let _: () = msg_send![panel, setIgnoresMouseEvents: YES];

    // Disable window animation (NSWindowAnimationBehaviorNone = 2)
    let _: () = msg_send![panel, setAnimationBehavior: 2i64];

    panel
}

/// Make the overlay visible.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn show_overlay(panel: *mut objc::runtime::Object) {
    let _: () = msg_send![panel, orderFrontRegardless];
}

/// Hide the overlay.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn hide_overlay(panel: *mut objc::runtime::Object) {
    let _: () = msg_send![panel, orderOut: nil];
}

/// Enable or disable click-through (mouse event passthrough).
/// `enabled = true`  → pointer mode (mouse passes through to underlying apps)
/// `enabled = false` → draw mode (mouse events captured for annotation)
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn set_click_through(panel: *mut objc::runtime::Object, enabled: bool) {
    let val: cocoa::base::BOOL = if enabled { YES } else { NO };
    let _: () = msg_send![panel, setIgnoresMouseEvents: val];
}

/// Resize and reposition the overlay to match a display's frame.
///
/// # Safety
/// Must be called on the main thread.
pub unsafe fn resize_overlay(
    panel: *mut objc::runtime::Object,
    x: f64, y: f64, width: f64, height: f64,
) {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};
    let frame = NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    };
    let _: () = msg_send![panel, setFrame: frame display: false];
}
