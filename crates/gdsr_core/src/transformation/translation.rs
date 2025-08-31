use crate::{CoordNum, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Translation<T: CoordNum> {
    pub delta: Point<T>,
}

impl<T: CoordNum> Translation<T> {
    pub fn new(delta: Point<T>) -> Self {
        Self { delta }
    }

    pub fn apply_to_point(&self, point: &Point<T>) -> Point<T> {
        Point::new(point.x() + self.delta.x(), point.y() + self.delta.y())
    }
}
