#[cfg(target_os = "macos")]
pub mod canvas;
#[cfg(target_os = "macos")]
pub mod draw;

use crate::state::SharedState;

/// Start the 120fps render loop on a dedicated background thread.
///
/// # Safety
/// `ns_panel` must remain valid for the lifetime of the app.
/// `MetalCanvas::new` must be called on a thread where the Metal context is valid —
/// background threads are fine for Metal rendering.
#[cfg(target_os = "macos")]
pub fn start_render_loop(
    ns_panel: *mut objc::runtime::Object,
    state: SharedState,
    width: i32,
    height: i32,
    scale: f32,
) {
    use canvas::MetalCanvas;
    use draw::draw_frame;
    use skia_safe::Canvas;

    // Escape the pointer through usize so it's Send across the thread boundary.
    // SAFETY: ns_panel lives for the duration of the app (owned by the OS).
    let panel_usize = ns_panel as usize;

    std::thread::spawn(move || {
        let ns_panel = panel_usize as *mut objc::runtime::Object;
        let mut metal_canvas = unsafe { MetalCanvas::new(ns_panel, width, height, scale) };

        let frame_budget = std::time::Duration::from_micros(8_333); // ~120fps

        loop {
            let frame_start = std::time::Instant::now();

            // --- Take a snapshot under a short lock ---
            let (visible, drawing_snapshot, cursor_pos) = {
                let s = state.lock();
                let snapshot = s.drawing.clone();
                let cursor = s.cursor_pos.clone();
                (s.overlay_visible, snapshot, cursor)
            };

            // cursor_pos is captured for future use (cursor effects in later tasks)
            let _ = cursor_pos;

            // --- Render only when overlay is visible ---
            if visible {
                let scale_copy = scale;
                metal_canvas.render_frame(|canvas: &Canvas| {
                    draw_frame(canvas, &drawing_snapshot, scale_copy);
                });
            }

            // --- Sleep remaining budget ---
            let elapsed = frame_start.elapsed();
            if elapsed < frame_budget {
                std::thread::sleep(frame_budget - elapsed);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn renderer_module_compiles() {
        assert!(true);
    }
}
