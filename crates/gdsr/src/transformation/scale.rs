use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    factor: f64,
    centre: Point,
}

impl Scale {
    pub const fn new(factor: f64, centre: Point) -> Self {
        Self { factor, centre }
    }

    pub const fn factor(&self) -> f64 {
        self.factor
    }

    pub const fn centre(&self) -> &Point {
        &self.centre
    }

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
    fn test_scale_half() {
        let scale = Scale::new(0.5, Point::integer(0, 0, 1e-9));
        let point = Point::integer(20, 10, 1e-9);
        let scaled = scale.apply_to_point(&point);

        // Point should be half as far from origin
        let expected = Point::integer(10, 5, 1e-9);
        assert_eq!(scaled.x(), expected.x());
        assert_eq!(scaled.y(), expected.y());
    }
}
