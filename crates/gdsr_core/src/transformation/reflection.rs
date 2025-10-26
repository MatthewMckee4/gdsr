use crate::{AngleInRadians, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Reflection {
    angle: AngleInRadians,
    centre: Point,
}

impl Reflection {
    #[must_use]
    pub const fn new(angle: AngleInRadians, centre: Point) -> Self {
        Self { angle, centre }
    }

    #[must_use]
    pub const fn new_horizontal() -> Self {
        Self::new(0.0, Point::integer(0, 1, 1e-9))
    }

    pub fn from_line(_point1: &Point, _point2: &Point) {
        todo!()
    }

    #[must_use]
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
    fn test_reflection_clone() {
        let reflection = Reflection::new(30.0, Point::integer(5, 5, 1e-9));
        let cloned = reflection.clone();
        assert_eq!(reflection, cloned);
    }

    #[test]
    fn test_reflection_apply_at_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let reflection = Reflection::new(45.0, centre);
        let reflected = reflection.apply_to_point(&centre);

        // Point at centre should remain unchanged
        assert_eq!(reflected, centre);
    }
}
