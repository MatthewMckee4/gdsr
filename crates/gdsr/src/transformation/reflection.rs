use crate::{AngleInRadians, Point};

/// A reflection transformation defined by an axis angle and centre point.
#[derive(Clone, Debug, PartialEq)]
pub struct Reflection {
    angle: AngleInRadians,
    centre: Point,
}

impl std::fmt::Display for Reflection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Reflection with angle {} rad about {}",
            self.angle, self.centre
        )
    }
}

impl Reflection {
    /// Creates a new reflection with the given axis angle (in radians) and centre point.
    pub const fn new(angle: AngleInRadians, centre: Point) -> Self {
        Self { angle, centre }
    }

    /// Creates a horizontal reflection (angle = 0) about the x-axis.
    pub const fn new_horizontal() -> Self {
        Self::new(0.0, Point::integer(0, 1, 1e-9))
    }

    /// Creates a reflection from a line defined by two points.
    pub fn from_line(point1: &Point, point2: &Point) -> Self {
        let dx = (point2.x() - point1.x()).absolute_value();
        let dy = (point2.y() - point1.y()).absolute_value();
        let angle = dy.atan2(dx);
        let centre = Point::new(
            (point1.x() + point2.x()) / 2.0,
            (point1.y() + point2.y()) / 2.0,
        );
        Self { angle, centre }
    }

    /// Reflects a point across this reflection's axis and returns the new point.
    pub fn apply_to_point(&self, point: &Point) -> Point {
        let cos_2angle = (2.0 * self.angle).cos();
        let sin_2angle = (2.0 * self.angle).sin();

        let self_center_x = self.centre.x();
        let self_center_y = self.centre.y();

        let dx = point.x() - self_center_x;
        let dy = point.y() - self_center_y;

        let new_x = (dx * cos_2angle) + (dy * sin_2angle) + self_center_x;
        let new_y = (dx * sin_2angle) - (dy * cos_2angle) + self_center_y;

        Point::new(new_x, new_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_new() {
        let reflection = Reflection::new(45.0, Point::integer(10, 20, 1e-9));
        assert_eq!(reflection.angle, 45.0);
        assert_eq!(reflection.centre, Point::integer(10, 20, 1e-9));
    }

    #[test]
    fn test_reflection_new_horizontal() {
        let reflection = Reflection::new_horizontal();
        assert_eq!(reflection.angle, 0.0);
        assert_eq!(reflection.centre, Point::integer(0, 1, 1e-9));
    }

    #[test]
    fn test_reflection_horizontal_axis() {
        let reflection = Reflection::new(0.0, Point::integer(0, 0, 1e-9));
        let point = Point::integer(10, 5, 1e-9);
        let reflected = reflection.apply_to_point(&point);

        // Reflection across horizontal axis (y=0) should flip y coordinate
        let expected_x = Point::integer(10, 0, 1e-9).x();
        let expected_y = Point::integer(0, -5, 1e-9).y();
        assert_eq!(reflected.x(), expected_x);
        assert_eq!(reflected.y(), expected_y);
    }

    #[test]
    fn test_reflection_apply_at_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let reflection = Reflection::new(45.0, centre);
        let reflected = reflection.apply_to_point(&centre);

        // Point at centre should remain unchanged
        assert_eq!(reflected, centre);
    }

    #[test]
    fn test_reflection_from_line_horizontal() {
        let point1 = Point::integer(0, 5, 1e-9);
        let point2 = Point::integer(10, 5, 1e-9);
        let reflection = Reflection::from_line(&point1, &point2);

        // Horizontal line should have angle 0
        assert_eq!(reflection.angle, 0.0);
        // Centre should be midpoint
        assert_eq!(reflection.centre, Point::integer(5, 5, 1e-9));
    }

    #[test]
    fn test_reflection_from_line_vertical() {
        let point1 = Point::integer(5, 0, 1e-9);
        let point2 = Point::integer(5, 10, 1e-9);
        let reflection = Reflection::from_line(&point1, &point2);

        // Vertical line should have angle π/2
        let expected_angle = std::f64::consts::FRAC_PI_2;
        assert!((reflection.angle - expected_angle).abs() < 1e-10);
        // Centre should be midpoint
        assert_eq!(reflection.centre, Point::integer(5, 5, 1e-9));
    }

    #[test]
    fn test_reflection_from_line_diagonal() {
        let point1 = Point::integer(0, 0, 1e-9);
        let point2 = Point::integer(10, 10, 1e-9);
        let reflection = Reflection::from_line(&point1, &point2);

        // Diagonal line (45 degrees) should have angle π/4
        let expected_angle = std::f64::consts::FRAC_PI_4;
        assert!((reflection.angle - expected_angle).abs() < 1e-10);
        // Centre should be midpoint
        assert_eq!(reflection.centre, Point::integer(5, 5, 1e-9));
    }

    #[test]
    fn test_reflection_display() {
        let reflection = Reflection::new(0.5, Point::integer(10, 20, 1e-9));
        insta::assert_snapshot!(reflection.to_string(), @"Reflection with angle 0.5 rad about Point(10 (1.000e-9), 20 (1.000e-9))");
    }

    #[test]
    fn test_reflection_from_line_apply() {
        let point1 = Point::integer(0, 0, 1e-9);
        let point2 = Point::integer(10, 0, 1e-9);
        let reflection = Reflection::from_line(&point1, &point2);

        let test_point = Point::integer(5, 3, 1e-9);
        let reflected = reflection.apply_to_point(&test_point);

        // Reflecting across horizontal line at y=0 should flip y
        assert_eq!(reflected.x(), test_point.x());
        let sum = (reflected.y() + test_point.y()).absolute_value();
        assert!(sum.abs() < 1e-9);
    }

    #[test]
    fn test_reflection_from_line_negative_quadrant() {
        let point1 = Point::integer(-5, -10, 1e-9);
        let point2 = Point::integer(0, 0, 1e-9);
        let reflection = Reflection::from_line(&point1, &point2);

        let expected_angle = (10.0_f64).atan2(5.0);
        assert!((reflection.angle - expected_angle).abs() < 1e-10);

        assert_eq!(reflection.centre, Point::integer(-3, -5, 1e-9));

        // Reflecting (1, 0) across the line from (-5, -10) to (0, 0)
        let test_point = Point::integer(1, 0, 1e-9);
        let reflected = reflection.apply_to_point(&test_point);
        let double_reflected = reflection.apply_to_point(&reflected);
        assert!(
            (double_reflected.x() - test_point.x())
                .absolute_value()
                .abs()
                < 1e-6
        );
        assert!(
            (double_reflected.y() - test_point.y())
                .absolute_value()
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn test_reflection_from_line_crossing_quadrants() {
        let point1 = Point::integer(-5, 3, 1e-9);
        let point2 = Point::integer(5, -3, 1e-9);
        let reflection = Reflection::from_line(&point1, &point2);

        let expected_angle = (-6.0_f64).atan2(10.0);
        assert!((reflection.angle - expected_angle).abs() < 1e-10);

        assert_eq!(reflection.centre, Point::integer(0, 0, 1e-9));

        // Reflecting (3, 3) across the line from (-5, 3) to (5, -3)
        let test_point = Point::integer(3, 3, 1e-9);
        let reflected = reflection.apply_to_point(&test_point);
        let double_reflected = reflection.apply_to_point(&reflected);
        assert!(
            (double_reflected.x() - test_point.x())
                .absolute_value()
                .abs()
                < 1e-6
        );
        assert!(
            (double_reflected.y() - test_point.y())
                .absolute_value()
                .abs()
                < 1e-6
        );
    }
}
