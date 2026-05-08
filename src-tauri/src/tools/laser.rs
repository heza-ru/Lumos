/// Compute alpha multiplier for a laser stroke given its age.
///
/// `age_ms`:  milliseconds elapsed since the stroke was created.
/// `fade_ms`: total fade duration in milliseconds.
///
/// Returns 1.0 when age is 0, decaying linearly to 0.0 at age == fade_ms.
/// Returns 0.0 for any age >= fade_ms.
pub fn laser_alpha(age_ms: u64, fade_ms: u64) -> f32 {
    if age_ms >= fade_ms {
        return 0.0;
    }
    1.0 - (age_ms as f32 / fade_ms as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha_at_zero_age_is_one() {
        assert!((laser_alpha(0, 2000) - 1.0).abs() < 0.01);
    }

    #[test]
    fn alpha_at_half_lifetime_is_half() {
        let a = laser_alpha(1000, 2000);
        assert!((a - 0.5).abs() < 0.01);
    }

    #[test]
    fn alpha_at_full_lifetime_is_zero() {
        assert!(laser_alpha(2000, 2000) < 0.01);
    }

    #[test]
    fn alpha_past_lifetime_is_zero() {
        assert_eq!(laser_alpha(3000, 2000), 0.0);
    }

    #[test]
    fn alpha_clamps_to_zero_not_negative() {
        assert!(laser_alpha(9999, 2000) >= 0.0);
    }
}
