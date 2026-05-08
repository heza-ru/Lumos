#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpotlightShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

impl Default for SpotlightShape {
    fn default() -> Self { SpotlightShape::Circle { radius: 120.0 } }
}

/// Draw a dim overlay with a spotlight hole cut at (cx, cy).
#[cfg(target_os = "macos")]
pub fn draw_spotlight(
    canvas: &skia_safe::Canvas,
    cx: f32,
    cy: f32,
    shape: SpotlightShape,
    dim_alpha: f32,
    screen_w: f32,
    screen_h: f32,
) {
    use skia_safe::{BlendMode, Color4f, Paint, PaintStyle, Rect};

    // 1. Dim overlay
    let mut dim = Paint::default();
    dim.set_color4f(Color4f::new(0.0, 0.0, 0.0, dim_alpha), None);
    dim.set_style(PaintStyle::Fill);
    dim.set_blend_mode(BlendMode::SrcOver);
    canvas.draw_rect(Rect::from_wh(screen_w, screen_h), &dim);

    // 2. Cut spotlight hole with Clear blend mode
    let mut hole = Paint::default();
    hole.set_blend_mode(BlendMode::Clear);
    hole.set_anti_alias(true);
    hole.set_style(PaintStyle::Fill);

    match shape {
        SpotlightShape::Circle { radius } => {
            canvas.draw_circle((cx, cy), radius, &hole);
        }
        SpotlightShape::Rectangle { width, height } => {
            let rect = Rect::from_xywh(cx - width / 2.0, cy - height / 2.0, width, height);
            canvas.draw_round_rect(rect, 12.0, 12.0, &hole);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spotlight_shape_default_is_circle() {
        matches!(SpotlightShape::default(), SpotlightShape::Circle { .. });
    }

    #[test]
    fn spotlight_shapes_can_be_constructed() {
        let _ = SpotlightShape::Circle { radius: 100.0 };
        let _ = SpotlightShape::Rectangle { width: 300.0, height: 200.0 };
    }
}
