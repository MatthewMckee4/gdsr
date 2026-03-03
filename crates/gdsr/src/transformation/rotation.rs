use crate::{AngleInRadians, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Rotation {
    angle: AngleInRadians,
    centre: Point,
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Rotation of {} rad about {}", self.angle, self.centre)
    }
}

impl Rotation {
    pub const fn new(angle: AngleInRadians, centre: Point) -> Self {
        Self { angle, centre }
    }

    pub const fn angle(&self) -> AngleInRadians {
        self.angle
    }

    pub const fn centre(&self) -> &Point {
        &self.centre
    }

    pub fn apply_to_point(&self, point: &Point) -> Point {
        let cos_angle = self.angle.cos();
        let sin_angle = self.angle.sin();

        let self_center_x = self.centre.x();
        let self_center_y = self.centre.y();

        let dx = point.x() - self_center_x;
        let dy = point.y() - self_center_y;

        let new_x = (dy * -sin_angle) + (dx * cos_angle) + self_center_x;
        let new_y = (dy * cos_angle) + (dx * sin_angle) + self_center_y;

        Point::new(new_x, new_y)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

    use super::*;

    #[test]
    fn test_rotation_new() {
        let rotation = Rotation::new(FRAC_PI_2, Point::integer(10, 20, 1e-9));
        assert_eq!(rotation.angle(), FRAC_PI_2);
        assert_eq!(rotation.centre(), &Point::integer(10, 20, 1e-9));
    }

    #[test]
    fn test_rotation_getters() {
        let centre = Point::integer(5, 10, 1e-9);
        let rotation = Rotation::new(FRAC_PI_4, centre);
        assert_eq!(rotation.angle(), FRAC_PI_4);
        assert_eq!(rotation.centre(), &centre);
    }

    #[test]
    fn test_rotation_clone() {
        let rotation = Rotation::new(30.0, Point::integer(5, 5, 1e-9));
        let cloned = rotation.clone();
        assert_eq!(rotation, cloned);
    }

    #[test]
    fn test_rotation_apply_at_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let rotation = Rotation::new(90.0, centre);
        let rotated = rotation.apply_to_point(&centre);

        // Point at centre should remain unchanged
        assert_eq!(rotated, centre);
    }

    #[test]
    fn test_rotation_display() {
        let rotation = Rotation::new(FRAC_PI_2, Point::integer(10, 20, 1e-9));
        insta::assert_snapshot!(rotation.to_string(), @"Rotation of 1.5707963267948966 rad about Point(10 (1.000e-9), 20 (1.000e-9))");
    }

    #[test]
    fn test_rotation_90_degrees() {
        let rotation = Rotation::new(90.0, Point::integer(0, 0, 1e-9));
        let point = Point::integer(10, 0, 1e-9);
        let rotated = rotation.apply_to_point(&point);

        // (10, 0) rotated 90° around origin should be approximately (0, 10)
        let expected = Point::integer(0, 10, 1e-9);
        // Allow small floating point errors
        assert!((rotated.x() - expected.x()).absolute_value().abs() < 1e-6);
        assert!((rotated.y() - expected.y()).absolute_value().abs() < 1e-6);
    }
}
