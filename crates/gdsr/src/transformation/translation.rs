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
}
