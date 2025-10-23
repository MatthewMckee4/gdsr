use crate::{AngleInDegrees, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Reflection {
    angle: AngleInDegrees,
    centre: Point,
}

impl Reflection {
    #[must_use]
    pub const fn new(angle: AngleInDegrees, centre: Point) -> Self {
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
