use crate::{CoordNum, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Scale<T: CoordNum> {
    pub factor: T,
    pub centre: Point<T>,
}

impl<T: CoordNum> Scale<T> {
    pub fn new(factor: T, centre: Point<T>) -> Self {
        Self { factor, centre }
    }

    pub fn apply_to_point(&self, point: &Point<T>) -> Point<T> {
        let dx = point.x() - self.centre.x();
        let dy = point.y() - self.centre.y();

        let new_x = self.centre.x() + dx * self.factor;
        let new_y = self.centre.y() + dy * self.factor;

        Point::new(new_x, new_y)
    }
}
