use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    factor: f64,
    centre: Point,
}

impl Scale {
    #[must_use]
    pub fn new(factor: f64, centre: impl Into<Point>) -> Self {
        Self {
            factor,
            centre: centre.into(),
        }
    }

    #[must_use]
    pub const fn factor(&self) -> f64 {
        self.factor
    }

    #[must_use]
    pub const fn centre(&self) -> &Point {
        &self.centre
    }

    #[must_use]
    pub fn apply_to_point(&self, point: &Point) -> Point {
        let self_center_x = self.centre.x();
        let self_center_y = self.centre.y();

        let dx = point.x() - self_center_x;
        let dy = point.y() - self_center_y;

        let new_x = (dx * self.factor) + self_center_x;
        let new_y = (dy * self.factor) + self_center_y;

        (new_x, new_y).into()
    }
}
