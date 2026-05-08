/// Logical coordinates and scale for one display.
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id:     u32,
    pub x:      f32,
    pub y:      f32,
    pub width:  f32,
    pub height: f32,
    pub scale:  f32,
}

impl DisplayInfo {
    pub fn physical_width(&self)  -> f32 { self.width  * self.scale }
    pub fn physical_height(&self) -> f32 { self.height * self.scale }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }
}

/// Return the display that contains the given cursor position.
/// Falls back to the main screen if no display matches.
#[cfg(target_os = "macos")]
pub fn display_for_point(cursor_x: f32, cursor_y: f32) -> DisplayInfo {
    use objc::{class, msg_send, sel, sel_impl};
    use cocoa::foundation::NSRect;

    unsafe {
        let screens: cocoa::base::id = msg_send![class!(NSScreen), screens];
        let count: usize = msg_send![screens, count];

        let mut primary = DisplayInfo { id: 0, x: 0.0, y: 0.0, width: 1920.0, height: 1080.0, scale: 2.0 };

        for i in 0..count {
            let screen: cocoa::base::id = msg_send![screens, objectAtIndex: i];
            let frame: NSRect = msg_send![screen, frame];
            let scale: f64 = msg_send![screen, backingScaleFactor];

            let info = DisplayInfo {
                id:     i as u32,
                x:      frame.origin.x as f32,
                y:      frame.origin.y as f32,
                width:  frame.size.width  as f32,
                height: frame.size.height as f32,
                scale:  scale as f32,
            };

            if i == 0 { primary = info.clone(); }

            if info.contains_point(cursor_x, cursor_y) {
                return info;
            }
        }
        primary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_info_physical_size() {
        let d = DisplayInfo { id: 0, x: 0.0, y: 0.0, width: 2560.0, height: 1440.0, scale: 2.0 };
        assert_eq!(d.physical_width(), 5120.0);
        assert_eq!(d.physical_height(), 2880.0);
    }

    #[test]
    fn point_inside_display() {
        let d = DisplayInfo { id: 0, x: 0.0, y: 0.0, width: 2560.0, height: 1440.0, scale: 2.0 };
        assert!(d.contains_point(100.0, 100.0));
        assert!(!d.contains_point(3000.0, 100.0));
        assert!(!d.contains_point(100.0, 2000.0));
    }

    #[test]
    fn point_on_boundary_is_inside() {
        let d = DisplayInfo { id: 0, x: 100.0, y: 200.0, width: 1920.0, height: 1080.0, scale: 1.0 };
        assert!(d.contains_point(100.0, 200.0)); // at origin corner
    }
}
