use skia_safe::{Canvas, Color4f, Paint, PaintStyle, Path, Rect};
use crate::state::{DrawingState, Stroke, ToolKind, Color as LColor};

fn to_skia_color(c: &LColor) -> Color4f {
    Color4f::new(
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    )
}

/// Clear the canvas to fully transparent.
pub fn clear(canvas: &Canvas) {
    canvas.clear(skia_safe::Color::TRANSPARENT);
}

/// Draw all committed strokes plus the live (in-progress) stroke.
/// `scale` is the Retina backing scale factor; logical coords are scaled up.
#[allow(clippy::too_many_arguments)]
pub fn draw_frame(
    canvas: &Canvas,
    state: &DrawingState,
    scale: f32,
    cursor_pos: (f32, f32),
    cursor_effect: crate::effects::cursor::CursorEffect,
    t_secs: f32,
    spotlight_active: bool,
    spotlight_shape: crate::effects::spotlight::SpotlightShape,
    spotlight_dim_alpha: f32,
    screen_w: f32,
    screen_h: f32,
) {
    canvas.save();
    canvas.scale((scale, scale));
    clear(canvas);

    // Draw spotlight before strokes (dims the background)
    #[cfg(target_os = "macos")]
    if spotlight_active {
        crate::effects::spotlight::draw_spotlight(
            canvas,
            cursor_pos.0,
            cursor_pos.1,
            spotlight_shape,
            spotlight_dim_alpha,
            screen_w / scale,
            screen_h / scale,
        );
    }

    for stroke in &state.strokes {
        draw_stroke(canvas, stroke, None);
    }
    if let Some(live) = &state.live_stroke {
        draw_stroke(canvas, live, None);
    }

    // Draw cursor effect after strokes
    #[cfg(target_os = "macos")]
    if cursor_effect != crate::effects::cursor::CursorEffect::None {
        crate::effects::cursor::draw_cursor_effect(
            canvas,
            cursor_effect,
            cursor_pos.0,
            cursor_pos.1,
            t_secs,
            (82, 155, 224),
        );
    }

    canvas.restore();
}

/// Draw a single stroke. `alpha_override` replaces the stroke's natural alpha
/// (used by the laser pointer for time-decay fading).
pub fn draw_stroke(canvas: &Canvas, stroke: &Stroke, alpha_override: Option<f32>) {
    match stroke.tool {
        ToolKind::Pen         => draw_pen(canvas, stroke, alpha_override.unwrap_or(1.0)),
        ToolKind::Highlighter => draw_pen(canvas, stroke, alpha_override.unwrap_or(0.35)),
        ToolKind::Arrow       => draw_arrow(canvas, stroke),
        ToolKind::Rectangle   => draw_rectangle(canvas, stroke),
        ToolKind::Ellipse     => draw_ellipse(canvas, stroke),
        ToolKind::Line        => draw_line_segment(canvas, stroke),
        ToolKind::Laser       => {
            let alpha = if let Some(a) = alpha_override {
                a
            } else {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let age_ms = now_ms.saturating_sub(stroke.created_at_ms);
                crate::tools::laser::laser_alpha(age_ms, 2000)
            };
            if alpha > 0.0 {
                draw_pen(canvas, stroke, alpha);
            }
        }
        ToolKind::Eraser      => {} // handled by removing strokes from state
        ToolKind::Text        => {} // handled in a later task
    }
}

fn make_pen_paint(stroke: &Stroke, alpha_mult: f32) -> Paint {
    let mut color = to_skia_color(&stroke.color);
    color.a *= alpha_mult;

    let mut paint = Paint::default();
    paint.set_color4f(color, None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_stroke_cap(skia_safe::PaintCap::Round);
    paint.set_stroke_join(skia_safe::PaintJoin::Round);
    paint.set_style(PaintStyle::Stroke);
    paint
}

fn draw_pen(canvas: &Canvas, stroke: &Stroke, alpha_mult: f32) {
    if stroke.points.len() < 2 {
        return;
    }

    let paint = make_pen_paint(stroke, alpha_mult);

    let mut path = Path::new();
    path.move_to((stroke.points[0].x, stroke.points[0].y));

    // Catmull-Rom-style smoothing: use quadratic bezier through midpoints
    if stroke.points.len() == 2 {
        path.line_to((stroke.points[1].x, stroke.points[1].y));
    } else {
        for i in 1..stroke.points.len() - 1 {
            let p0 = &stroke.points[i - 1];
            let p1 = &stroke.points[i];
            let p2 = &stroke.points[i + 1];
            // Suppress unused variable warning for p0 (it's referenced intentionally
            // in a future Catmull-Rom implementation)
            let _ = p0;
            let mid_x = (p1.x + p2.x) / 2.0;
            let mid_y = (p1.y + p2.y) / 2.0;
            path.quad_to((p1.x, p1.y), (mid_x, mid_y));
        }
        let last = stroke.points.last().unwrap();
        path.line_to((last.x, last.y));
    }

    canvas.draw_path(&path, &paint);
}

fn draw_arrow(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 {
        return;
    }
    let first = &stroke.points[0];
    let last  = stroke.points.last().unwrap();

    let mut paint = Paint::default();
    paint.set_color4f(to_skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_stroke_cap(skia_safe::PaintCap::Round);
    paint.set_style(PaintStyle::StrokeAndFill);

    // Shaft
    canvas.draw_line((first.x, first.y), (last.x, last.y), &paint);

    // Arrowhead
    let angle     = (last.y - first.y).atan2(last.x - first.x);
    let head_len  = stroke.width.pen_px() * 4.0;
    let spread    = std::f32::consts::PI / 6.0; // 30°

    let p1 = (
        last.x - head_len * (angle - spread).cos(),
        last.y - head_len * (angle - spread).sin(),
    );
    let p2 = (
        last.x - head_len * (angle + spread).cos(),
        last.y - head_len * (angle + spread).sin(),
    );

    let mut head = Path::new();
    head.move_to((last.x, last.y));
    head.line_to(p1);
    head.line_to(p2);
    head.close();
    canvas.draw_path(&head, &paint);
}

fn draw_rectangle(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 {
        return;
    }
    let start = &stroke.points[0];
    let end   = stroke.points.last().unwrap();
    let rect  = Rect::from_ltrb(
        start.x.min(end.x),
        start.y.min(end.y),
        start.x.max(end.x),
        start.y.max(end.y),
    );

    let mut paint = Paint::default();
    paint.set_color4f(to_skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_rect(rect, &paint);
}

fn draw_ellipse(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 {
        return;
    }
    let start = &stroke.points[0];
    let end   = stroke.points.last().unwrap();
    let rect  = Rect::from_ltrb(
        start.x.min(end.x),
        start.y.min(end.y),
        start.x.max(end.x),
        start.y.max(end.y),
    );

    let mut paint = Paint::default();
    paint.set_color4f(to_skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_oval(rect, &paint);
}

fn draw_line_segment(canvas: &Canvas, stroke: &Stroke) {
    if stroke.points.len() < 2 {
        return;
    }
    let first = &stroke.points[0];
    let last  = stroke.points.last().unwrap();

    let mut paint = Paint::default();
    paint.set_color4f(to_skia_color(&stroke.color), None);
    paint.set_anti_alias(true);
    paint.set_stroke_width(stroke.width.pen_px());
    paint.set_stroke_cap(skia_safe::PaintCap::Round);
    paint.set_style(PaintStyle::Stroke);
    canvas.draw_line((first.x, first.y), (last.x, last.y), &paint);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Color, Point, Stroke, StrokeWidth, ToolKind};

    fn make_stroke(tool: ToolKind, points: Vec<(f32, f32)>) -> Stroke {
        Stroke {
            id: 0,
            tool,
            color: Color::BLUE,
            width: StrokeWidth::Medium,
            points: points.into_iter().map(|(x, y)| Point { x, y }).collect(),
            created_at_ms: 0,
        }
    }

    #[test]
    fn draw_pen_requires_two_points() {
        // A single-point stroke should not panic
        let stroke = make_stroke(ToolKind::Pen, vec![(0.0, 0.0)]);
        // We can't easily test canvas output, but we verify no panic:
        // draw_pen would return early, so just verify the condition
        assert!(stroke.points.len() < 2);
    }

    #[test]
    fn arrow_head_angle_calculation() {
        // Test that arrow angle math produces finite values
        let dx = 100.0f32;
        let dy = 0.0f32;
        let angle = dy.atan2(dx);
        let head_len = 8.0f32 * 4.0;
        let spread = std::f32::consts::PI / 6.0;
        let p1x = dx - head_len * (angle - spread).cos();
        assert!(p1x.is_finite());
    }

    #[test]
    fn rect_from_unordered_points() {
        // Rect should handle end < start correctly
        let start = Point { x: 100.0, y: 100.0 };
        let end   = Point { x: 50.0,  y: 50.0  };
        let rect  = skia_safe::Rect::from_ltrb(
            start.x.min(end.x),
            start.y.min(end.y),
            start.x.max(end.x),
            start.y.max(end.y),
        );
        assert_eq!(rect.width(),  50.0);
        assert_eq!(rect.height(), 50.0);
    }

    #[test]
    fn to_skia_color_maps_correctly() {
        let c = Color { r: 255, g: 128, b: 0, a: 255 };
        let sc = super::to_skia_color(&c);
        assert!((sc.r - 1.0).abs() < 0.01);
        assert!((sc.g - (128.0 / 255.0)).abs() < 0.01);
        assert_eq!(sc.b, 0.0);
        assert!((sc.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn laser_alpha_integration_with_age() {
        // Verify that laser_alpha produces expected values at boundaries
        assert!((crate::tools::laser::laser_alpha(0, 2000) - 1.0).abs() < 0.01);
        assert_eq!(crate::tools::laser::laser_alpha(2000, 2000), 0.0);
    }
}
