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
        let loop_start = std::time::Instant::now();

        loop {
            let frame_start = std::time::Instant::now();
            let t_secs = loop_start.elapsed().as_secs_f32();

            // --- Take a snapshot under a short lock ---
            let (visible, drawing_snapshot, cursor_pos, cursor_effect, spotlight_active, spotlight_shape, spotlight_dim_alpha) = {
                let s = state.lock();
                let snapshot = s.drawing.clone();
                let cursor = (s.cursor_pos.x, s.cursor_pos.y);
                let effect = s.cursor_effect;
                let spot_active = s.spotlight_active;
                let spot_shape = s.spotlight_shape;
                let spot_dim = s.spotlight_dim_alpha;
                (s.overlay_visible, snapshot, cursor, effect, spot_active, spot_shape, spot_dim)
            };

            // --- Render only when overlay is visible ---
            if visible {
                let scale_copy = scale;
                let screen_w = width as f32;
                let screen_h = height as f32;
                metal_canvas.render_frame(|canvas: &Canvas| {
                    draw_frame(
                        canvas,
                        &drawing_snapshot,
                        scale_copy,
                        cursor_pos,
                        cursor_effect,
                        t_secs,
                        spotlight_active,
                        spotlight_shape,
                        spotlight_dim_alpha,
                        screen_w,
                        screen_h,
                    );
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
