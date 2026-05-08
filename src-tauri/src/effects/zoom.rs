/// Source rect (x, y, w, h) to sample for zoom lens magnification.
pub fn zoom_source_rect(cx: f32, cy: f32, zoom_factor: f32, lens_diameter: f32) -> (f32, f32, f32, f32) {
    let source_size = lens_diameter / zoom_factor;
    let x = cx - source_size / 2.0;
    let y = cy - source_size / 2.0;
    (x, y, source_size, source_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_source_rect_centered_on_cursor() {
        let (x, y, w, h) = zoom_source_rect(100.0, 100.0, 2.0, 80.0);
        // source_size = 80 / 2 = 40
        assert!((x - 80.0).abs() < 0.01);  // 100 - 40/2
        assert!((y - 80.0).abs() < 0.01);
        assert!((w - 40.0).abs() < 0.01);
        assert!((h - 40.0).abs() < 0.01);
    }

    #[test]
    fn zoom_source_rect_3x_zoom() {
        let (_x, _y, w, h) = zoom_source_rect(50.0, 50.0, 3.0, 150.0);
        // source_size = 150 / 3 = 50
        assert!((w - 50.0).abs() < 0.01);
        assert!((h - 50.0).abs() < 0.01);
    }

    #[test]
    fn zoom_source_rect_larger_zoom_means_smaller_source() {
        let (_, _, w1, _) = zoom_source_rect(100.0, 100.0, 2.0, 100.0);
        let (_, _, w2, _) = zoom_source_rect(100.0, 100.0, 4.0, 100.0);
        assert!(w1 > w2, "higher zoom should capture smaller source area");
    }
}
