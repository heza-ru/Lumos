use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub laser_fade_ms:          u64,
    pub cursor_effect_enabled:  bool,
    pub spotlight_dim_alpha:    f32,
    pub zoom_factor:            f32,
    pub hotkey_toggle_overlay:  String,
    pub hotkey_clear:           String,
    pub hotkey_undo:            String,
    pub launch_hidden:          bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            laser_fade_ms:         2000,
            cursor_effect_enabled: true,
            spotlight_dim_alpha:   0.65,
            zoom_factor:           2.5,
            hotkey_toggle_overlay: "CmdOrCtrl+Shift+A".into(),
            hotkey_clear:          "CmdOrCtrl+K".into(),
            hotkey_undo:           "CmdOrCtrl+Z".into(),
            launch_hidden:         false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let s = Settings::default();
        assert_eq!(s.laser_fade_ms, 2000);
        assert_eq!(s.spotlight_dim_alpha, 0.65);
        assert!(s.cursor_effect_enabled);
    }

    #[test]
    fn settings_serializes_to_json() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("laser_fade_ms"));
    }

    #[test]
    fn settings_round_trips_json() {
        let original = Settings::default();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.laser_fade_ms, original.laser_fade_ms);
        assert_eq!(restored.zoom_factor, original.zoom_factor);
    }
}
