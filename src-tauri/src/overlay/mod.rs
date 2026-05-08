#[cfg(target_os = "macos")]
mod panel;

#[cfg(target_os = "macos")]
pub use panel::{create_overlay, set_click_through, show_overlay, hide_overlay};

/// Opaque handle to the platform overlay window.
/// Wraps the raw NSPanel pointer so it can be stored in Tauri managed state.
pub struct OverlayHandle {
    #[cfg(target_os = "macos")]
    pub ns_panel: *mut objc::runtime::Object,
}

// SAFETY: NSPanel pointer is only accessed on the main thread via Tauri's
//         main-thread executor. We assert this invariant at call sites.
#[cfg(target_os = "macos")]
unsafe impl Send for OverlayHandle {}
#[cfg(target_os = "macos")]
unsafe impl Sync for OverlayHandle {}

/// Thread-safe reference to the overlay stored in Tauri managed state.
pub struct OverlayRef(pub std::sync::Mutex<*mut objc::runtime::Object>);

#[cfg(target_os = "macos")]
unsafe impl Send for OverlayRef {}
#[cfg(target_os = "macos")]
unsafe impl Sync for OverlayRef {}

impl OverlayRef {
    pub fn new(panel: *mut objc::runtime::Object) -> Self {
        Self(std::sync::Mutex::new(panel))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn overlay_module_compiles() {
        assert!(true);
    }
}
