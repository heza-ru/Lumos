use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorEffect {
    #[default]
    None,
    Glow,
    Ring,
    Pulse,
}

/// Radius for the pulsing cursor ring (oscillates between base and base+amplitude at 1 Hz).
pub fn pulse_radius(t_secs: f32, base: f32, amplitude: f32) -> f32 {
    base + amplitude * (0.5 + 0.5 * (TAU * t_secs).cos())
}

/// Alpha for a click ripple effect (linear decay from 1.0→0.0 over 1 second).
pub fn ripple_alpha(t_norm: f32) -> f32 {
    1.0 - t_norm.clamp(0.0, 1.0)
}

/// Draw the cursor effect at (cx, cy).
/// `t_secs`: seconds since app start (for animations).
/// `color`: (r, g, b) u8 values.
#[cfg(target_os = "macos")]
pub fn draw_cursor_effect(
    canvas: &skia_safe::Canvas,
    effect: CursorEffect,
    cx: f32,
    cy: f32,
    t_secs: f32,
    color: (u8, u8, u8),
) {
    use skia_safe::{Color4f, Paint, PaintStyle};

    if effect == CursorEffect::None { return; }

    let r = color.0 as f32 / 255.0;
    let g = color.1 as f32 / 255.0;
    let b = color.2 as f32 / 255.0;

    match effect {
        CursorEffect::None => {}

        CursorEffect::Glow => {
            for i in 0u8..3 {
                let radius = 16.0 + i as f32 * 10.0;
                let alpha = 0.15 - i as f32 * 0.04;
                let mut paint = Paint::default();
                paint.set_color4f(Color4f::new(r, g, b, alpha), None);
                paint.set_anti_alias(true);
                paint.set_style(PaintStyle::Fill);
                canvas.draw_circle((cx, cy), radius, &paint);
            }
        }

        CursorEffect::Ring => {
            let mut paint = Paint::default();
            paint.set_color4f(Color4f::new(r, g, b, 0.8), None);
            paint.set_anti_alias(true);
            paint.set_stroke_width(2.5);
            paint.set_style(PaintStyle::Stroke);
            canvas.draw_circle((cx, cy), 18.0, &paint);
        }

        CursorEffect::Pulse => {
            let radius = pulse_radius(t_secs, 14.0, 8.0);
            let alpha = 0.6 - (radius - 14.0) / 8.0 * 0.4;
            let mut paint = Paint::default();
            paint.set_color4f(Color4f::new(r, g, b, alpha.max(0.0)), None);
            paint.set_anti_alias(true);
            paint.set_style(PaintStyle::Fill);
            canvas.draw_circle((cx, cy), radius, &paint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_radius_oscillates() {
        let r0 = pulse_radius(0.0, 20.0, 8.0);
        let r1 = pulse_radius(0.25, 20.0, 8.0);
        let r2 = pulse_radius(1.0, 20.0, 8.0);
        assert!((r0 - r2).abs() < 0.01, "should complete one full cycle in 1s");
        assert!((r0 - r1).abs() > 0.01, "should vary within cycle");
    }

    #[test]
    fn pulse_radius_stays_within_range() {
        for i in 0..100 {
            let t = i as f32 * 0.1;
            let r = pulse_radius(t, 14.0, 8.0);
            assert!(r >= 14.0 && r <= 22.0, "r={} out of [14, 22]", r);
        }
    }

    #[test]
    fn ripple_alpha_at_zero_is_one() {
        assert!((ripple_alpha(0.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn ripple_alpha_at_one_is_zero() {
        assert!((ripple_alpha(1.0)).abs() < 0.01);
    }

    #[test]
    fn ripple_alpha_decays_monotonically() {
        let a0 = ripple_alpha(0.0);
        let a1 = ripple_alpha(0.5);
        let a2 = ripple_alpha(1.0);
        assert!(a0 > a1);
        assert!(a1 > a2);
    }

    #[test]
    fn ripple_alpha_never_negative() {
        assert!(ripple_alpha(1.5) >= 0.0);
        assert!(ripple_alpha(100.0) >= 0.0);
    }
}
