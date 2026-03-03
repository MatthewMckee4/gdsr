use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Translation {
    delta: Point,
}

impl std::fmt::Display for Translation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Translation by {}", self.delta)
    }
}

impl Translation {
    pub const fn new(delta: Point) -> Self {
        Self { delta }
    }

    pub fn apply_to_point(&self, point: &Point) -> Point {
        point + self.delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_display() {
        let translation = Translation::new(Point::integer(5, -3, 1e-9));
        let display_str = format!("{translation}");
        assert!(display_str.contains("Translation by"));
        assert!(display_str.contains("Point("));
    }

    #[test]
    fn test_translation_new() {
        let delta = Point::integer(10, 20, 1e-9);
        let translation = Translation::new(delta);
        assert_eq!(translation.delta, delta);
    }

    #[test]
    fn test_translation_apply_positive_delta() {
        let translation = Translation::new(Point::integer(3, 7, 1e-9));
        let point = Point::integer(10, 20, 1e-9);
        let result = translation.apply_to_point(&point);
        assert_eq!(result, Point::integer(13, 27, 1e-9));
    }

    #[test]
    fn test_translation_apply_negative_delta() {
        let translation = Translation::new(Point::integer(-5, -10, 1e-9));
        let point = Point::integer(10, 20, 1e-9);
        let result = translation.apply_to_point(&point);
        assert_eq!(result, Point::integer(5, 10, 1e-9));
    }

    #[test]
    fn test_translation_zero_delta() {
        let translation = Translation::new(Point::integer(0, 0, 1e-9));
        let point = Point::integer(10, 20, 1e-9);
        let result = translation.apply_to_point(&point);
        assert_eq!(result, point);
    }

    #[test]
    fn test_translation_large_values() {
        let translation = Translation::new(Point::integer(1_000_000, -1_000_000, 1e-9));
        let point = Point::integer(500_000, 500_000, 1e-9);
        let result = translation.apply_to_point(&point);
        assert_eq!(result, Point::integer(1_500_000, -500_000, 1e-9));
    }

    #[test]
    fn test_translation_clone() {
        let translation = Translation::new(Point::integer(5, 5, 1e-9));
        let cloned = translation.clone();
        assert_eq!(translation, cloned);
    }
}
