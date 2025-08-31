use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Reflection {
    pub angle: f64,
    pub centre: Point,
}

impl Reflection {
    pub fn new(angle: f64, centre: Point) -> Self {
        Self { angle, centre }
    }

    pub fn apply_to_point(&self, point: &Point) -> Point {
        let cos_2angle = (2.0 * self.angle).cos();
        let sin_2angle = (2.0 * self.angle).sin();
        let dx = point.x() - self.centre.x();
        let dy = point.y() - self.centre.y();

        let new_x = self.centre.x() + dx * cos_2angle + dy * sin_2angle;
        let new_y = self.centre.y() + dx * sin_2angle - dy * cos_2angle;

        Point::new(new_x, new_y)
    }
}