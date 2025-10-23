use crate::{AngleInDegrees, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Rotation {
    angle: AngleInDegrees,
    centre: Point,
}

impl Rotation {
    #[must_use]
    pub fn new(angle: AngleInDegrees, centre: impl Into<Point>) -> Self {
        Self {
            angle,
            centre: centre.into(),
        }
    }

    #[must_use]
    pub const fn angle(&self) -> AngleInDegrees {
        self.angle
    }

    #[must_use]
    pub const fn centre(&self) -> &Point {
        &self.centre
    }

    #[must_use]
    pub fn apply_to_point(&self, point: &Point) -> Point {
        let cos_angle = self.angle.cos();
        let sin_angle = self.angle.sin();

        let self_center_x = self.centre.x();
        let self_center_y = self.centre.y();

        let dx = point.x() - self_center_x;
        let dy = point.y() - self_center_y;

        let new_x = (dy * -sin_angle) + (dx * cos_angle) + self_center_x;
        let new_y = (dy * cos_angle) + (dx * sin_angle) + self_center_y;

        (new_x, new_y).into()
    }
}
