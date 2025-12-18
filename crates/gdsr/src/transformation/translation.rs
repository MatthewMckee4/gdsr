use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Translation {
    delta: Point,
}

impl Translation {
    pub const fn new(delta: Point) -> Self {
        Self { delta }
    }

    pub fn apply_to_point(&self, point: &Point) -> Point {
        point + self.delta
    }
}
