use crate::Point;

/// A scale transformation defined by a factor and a centre point.
#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    factor: f64,
    centre: Point,
}

impl std::fmt::Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scale by {} about {}", self.factor, self.centre)
    }
}

impl Scale {
    /// Creates a new scale with the given factor and centre point.
    pub const fn new(factor: f64, centre: Point) -> Self {
        Self { factor, centre }
    }

    /// Returns the scale factor.
    pub const fn factor(&self) -> f64 {
        self.factor
    }

    /// Returns the centre point of the scale.
    pub const fn centre(&self) -> &Point {
        &self.centre
    }

    /// Scales a point relative to this scale's centre and returns the new point.
    pub fn apply_to_point(&self, point: &Point) -> Point {
        let self_center_x = self.centre.x();
        let self_center_y = self.centre.y();

        let dx = point.x() - self_center_x;
        let dy = point.y() - self_center_y;

        let new_x = (dx * self.factor) + self_center_x;
        let new_y = (dy * self.factor) + self_center_y;

        Point::new(new_x, new_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_new() {
        let scale = Scale::new(2.0, Point::integer(10, 20, 1e-9));
        assert_eq!(scale.factor(), 2.0);
        assert_eq!(scale.centre(), &Point::integer(10, 20, 1e-9));
    }

    #[test]
    fn test_scale_getters() {
        let centre = Point::integer(5, 10, 1e-9);
        let scale = Scale::new(1.5, centre);
        assert_eq!(scale.factor(), 1.5);
        assert_eq!(scale.centre(), &centre);
    }

    #[test]
    fn test_scale_clone() {
        let scale = Scale::new(3.0, Point::integer(5, 5, 1e-9));
        let cloned = scale.clone();
        assert_eq!(scale, cloned);
    }

    #[test]
    fn test_scale_apply_at_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let scale = Scale::new(2.0, centre);
        let scaled = scale.apply_to_point(&centre);

        // Point at centre should remain unchanged
        assert_eq!(scaled, centre);
    }

    #[test]
    fn test_scale_double() {
        let scale = Scale::new(2.0, Point::integer(0, 0, 1e-9));
        let point = Point::integer(10, 5, 1e-9);
        let scaled = scale.apply_to_point(&point);

        // Point should be twice as far from origin
        let expected = Point::integer(20, 10, 1e-9);
        assert_eq!(scaled.x(), expected.x());
        assert_eq!(scaled.y(), expected.y());
    }

    #[test]
    fn test_scale_display() {
        let scale = Scale::new(2.5, Point::integer(10, 20, 1e-9));
        insta::assert_snapshot!(scale.to_string(), @"Scale by 2.5 about Point(10 (1.000e-9), 20 (1.000e-9))");
    }

    #[test]
    fn test_scale_half() {
        let scale = Scale::new(0.5, Point::integer(0, 0, 1e-9));
        let point = Point::integer(20, 10, 1e-9);
        let scaled = scale.apply_to_point(&point);

        // Point should be half as far from origin
        let expected = Point::integer(10, 5, 1e-9);
        assert_eq!(scaled.x(), expected.x());
        assert_eq!(scaled.y(), expected.y());
    }

    #[test]
    fn test_scale_by_zero() {
        let scale = Scale::new(0.0, Point::integer(0, 0, 1e-9));
        let point = Point::integer(10, 20, 1e-9);
        let scaled = scale.apply_to_point(&point);

        assert_eq!(scaled.x().absolute_value(), 0.0);
        assert_eq!(scaled.y().absolute_value(), 0.0);
    }

    #[test]
    fn test_scale_by_zero_nonorigin_centre() {
        let centre = Point::integer(5, 5, 1e-9);
        let scale = Scale::new(0.0, centre);
        let point = Point::integer(10, 20, 1e-9);
        let scaled = scale.apply_to_point(&point);

        assert_eq!(scaled.x(), centre.x());
        assert_eq!(scaled.y(), centre.y());
    }

    #[test]
    fn test_scale_negative() {
        let scale = Scale::new(-1.0, Point::integer(0, 0, 1e-9));
        let point = Point::integer(10, 5, 1e-9);
        let scaled = scale.apply_to_point(&point);

        let expected = Point::integer(-10, -5, 1e-9);
        assert_eq!(scaled.x(), expected.x());
        assert_eq!(scaled.y(), expected.y());
    }

    #[test]
    fn test_scale_negative_with_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let scale = Scale::new(-2.0, centre);
        let point = Point::integer(15, 20, 1e-9);
        let scaled = scale.apply_to_point(&point);

        // (15-10)*-2 + 10 = 0, (20-10)*-2 + 10 = -10
        let expected = Point::integer(0, -10, 1e-9);
        assert_eq!(scaled.x(), expected.x());
        assert_eq!(scaled.y(), expected.y());
    }
}
