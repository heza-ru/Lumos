#![cfg(target_os = "macos")]

use crate::state::SharedState;

pub struct EventTap;

impl EventTap {
    /// Install the CGEventTap on a background thread.
    /// The tap is always active. `click_through` in SharedState
    /// controls whether events are consumed (draw) or passed through (pointer).
    pub fn install(state: SharedState) -> Self {
        std::thread::spawn(move || Self::run_tap(state));
        Self
    }

    fn run_tap(state: SharedState) {
        use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
        use core_graphics::event::{
            CallbackResult, CGEventTap, CGEventTapLocation, CGEventTapOptions,
            CGEventTapPlacement, CGEventType,
        };

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
                let location = event.location();
                let x = location.x as f32;
                let y = location.y as f32;

                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                let mut s = state.lock();

                // Always track cursor position (used by cursor glow / ring effects)
                s.cursor_pos = crate::state::Point { x, y };

                // Pointer mode: pass every event through unchanged
                if s.click_through {
                    return CallbackResult::Keep;
                }

                // Draw mode: record strokes and consume the event
                match event_type {
                    CGEventType::LeftMouseDown    => s.drawing.begin_stroke(x, y, ts),
                    CGEventType::LeftMouseDragged => s.drawing.extend_stroke(x, y),
                    CGEventType::LeftMouseUp      => s.drawing.commit_stroke(),
                    CGEventType::MouseMoved       => {} // cursor pos already updated
                    _                             => {}
                }

                CallbackResult::Drop
            },
        );

        match tap_result {
            Ok(tap) => {
                let source = tap
                    .mach_port()
                    .create_runloop_source(0)
                    .expect("failed to create run loop source");
                let rl = CFRunLoop::get_current();
                rl.add_source(&source, unsafe { kCFRunLoopCommonModes });
                tap.enable();
                CFRunLoop::run_current();
            }
            Err(_) => {
                log::warn!(
                    "CGEventTap failed — grant Accessibility permission in System Settings → Privacy"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn event_tap_module_compiles() {
        assert!(true);
    }
}
