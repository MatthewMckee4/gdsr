use crate::{CoordNum, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug, PartialEq)]
struct RotationInner<DatabaseUnitT: CoordNum, AngleT: CoordNum> {
    angle: AngleT,
    centre: Point<DatabaseUnitT>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rotation(RotationInner<DatabaseIntegerUnit, f64>);

impl Rotation {
    #[must_use]
    pub const fn new(angle: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        Self(RotationInner { angle, centre })
    }

    #[must_use]
    pub const fn angle(&self) -> f64 {
        self.0.angle
    }

    #[must_use]
    pub const fn centre(&self) -> &Point<DatabaseIntegerUnit> {
        &self.0.centre
    }

    pub fn apply_to_point<DatabaseUnitT: CoordNum>(
        &self,
        point: &Point<DatabaseUnitT>,
    ) -> Point<DatabaseUnitT> {
        let cos_angle = self.0.angle.cos();
        let sin_angle = self.0.angle.sin();

        let self_center_x = self.0.centre.x() as f64;
        let self_center_y = self.0.centre.y() as f64;

        let dx = (point.x().to_float()) - self_center_x;
        let dy = (point.y().to_float()) - self_center_y;

        let new_x = dy.mul_add(-sin_angle, dx.mul_add(cos_angle, self_center_x));
        let new_y = dy.mul_add(cos_angle, dx.mul_add(sin_angle, self_center_y));

        Point::new(
            DatabaseUnitT::from_float(new_x),
            DatabaseUnitT::from_float(new_y),
        )
    }
}
