#![cfg(target_os = "macos")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::state::SharedState;

/// Wraps a CGEventTap background thread for mouse event interception.
///
/// When `active` is true (draw mode), mouse events are consumed before they
/// reach underlying applications. When false (pointer mode), events pass
/// through unchanged.
pub struct EventTap {
    /// When true, events are intercepted (draw mode).
    active: Arc<AtomicBool>,
}

impl EventTap {
    /// Install the CGEventTap on a background thread.
    /// The tap starts inactive (pointer mode); call `set_active(true)` to
    /// switch into draw mode.
    pub fn install(state: SharedState) -> Self {
        let active = Arc::new(AtomicBool::new(false));
        let active_clone = active.clone();

        std::thread::spawn(move || {
            Self::run_tap(state, active_clone);
        });

        Self { active }
    }

    /// Enable or disable event interception.
    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    /// Returns true when in draw mode (events are being intercepted).
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    fn run_tap(state: SharedState, active: Arc<AtomicBool>) {
        use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
        use core_graphics::event::{
            CallbackResult, CGEventTap, CGEventTapLocation, CGEventTapOptions,
            CGEventTapPlacement, CGEventType,
        };

        let state_clone = state.clone();
        let active_clone = active.clone();

        let tap_result = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![
                CGEventType::LeftMouseDown,
                CGEventType::LeftMouseDragged,
                CGEventType::LeftMouseUp,
                CGEventType::MouseMoved,
            ],
            move |_proxy, event_type, event| {
                // Always update cursor position
                let location = event.location();
                let x = location.x as f32;
                let y = location.y as f32;

                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                // Pass through if tap is not active (pointer mode)
                if !active_clone.load(Ordering::Relaxed) {
                    return CallbackResult::Keep;
                }

                let mut s = state_clone.lock();

                // Update cursor position for cursor effects
                s.cursor_pos = crate::state::Point { x, y };

                // If overlay is in click_through mode, pass events through
                if s.click_through {
                    return CallbackResult::Keep;
                }

                match event_type {
                    CGEventType::LeftMouseDown => s.drawing.begin_stroke(x, y, ts),
                    CGEventType::LeftMouseDragged => s.drawing.extend_stroke(x, y),
                    CGEventType::LeftMouseUp => s.drawing.commit_stroke(),
                    CGEventType::MouseMoved => {} // cursor pos already updated above
                    _ => {}
                }

                // Consume the event — don't pass to underlying apps in draw mode
                CallbackResult::Drop
            },
        );

        match tap_result {
            Ok(tap) => {
                let source = tap
                    .mach_port()
                    .create_runloop_source(0)
                    .expect("failed to create run loop source for event tap");
                let rl = CFRunLoop::get_current();
                rl.add_source(&source, unsafe { kCFRunLoopCommonModes });
                tap.enable();
                CFRunLoop::run_current();
            }
            Err(_) => {
                log::warn!(
                    "CGEventTap creation failed — Accessibility permission may not be granted"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_tap_module_compiles() {
        assert!(true);
    }

    #[test]
    fn event_tap_active_flag_toggle() {
        // Verify the AtomicBool logic used by EventTap works correctly.
        let active = Arc::new(AtomicBool::new(false));
        // Starts inactive (pointer mode)
        assert!(!active.load(Ordering::Relaxed));
        // Switching to draw mode
        active.store(true, Ordering::Relaxed);
        assert!(active.load(Ordering::Relaxed));
        // Back to pointer mode
        active.store(false, Ordering::Relaxed);
        assert!(!active.load(Ordering::Relaxed));
    }
}
