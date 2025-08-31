use crate::{CoordNum, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Rotation<T: CoordNum> {
    pub angle: T,
    pub centre: Point<T>,
}

impl<T: CoordNum> Rotation<T> {
    pub fn new(angle: T, centre: Point<T>) -> Self {
        Self { angle, centre }
    }

    pub fn apply_to_point(&self, point: &Point<T>) -> Point<T> {
        let cos_angle = cos(self.angle);
        let sin_angle = sin(self.angle);
        let dx = point.x() - self.centre.x();
        let dy = point.y() - self.centre.y();

        let new_x = self.centre.x() + dx * cos_angle - dy * sin_angle;
        let new_y = self.centre.y() + dx * sin_angle + dy * cos_angle;

        Point::new(new_x, new_y)
    }
}
