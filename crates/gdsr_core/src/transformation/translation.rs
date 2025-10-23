use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Translation {
    delta: Point,
}

impl Translation {
    #[must_use]
    pub fn new(delta: impl Into<Point>) -> Self {
        Self {
            delta: delta.into(),
        }
    }

    #[must_use]
    pub fn apply_to_point(&self, point: &Point) -> Point {
        Point::new(point.x() + self.delta.x(), point.y() + self.delta.y())
    }
}
