use crate::{CoordNum, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug)]
struct RotationInner<DatabaseUnitT: CoordNum, AngleT: CoordNum> {
    angle: AngleT,
    centre: Point<DatabaseUnitT>,
}

#[derive(Clone, Debug)]
pub struct Rotation(RotationInner<DatabaseIntegerUnit, f64>);

impl Rotation {
    pub fn new(angle: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        Self(RotationInner { angle, centre })
    }

    pub fn angle(&self) -> f64 {
        self.0.angle
    }

    pub fn centre(&self) -> &Point<DatabaseIntegerUnit> {
        &self.0.centre
    }

    pub fn apply_to_point<DatabaseUnitT: CoordNum>(
        &self,
        point: &Point<DatabaseUnitT>,
    ) -> Point<DatabaseUnitT> {
        let cos_angle = (self.0.angle as f64).cos();
        let sin_angle = (self.0.angle as f64).sin();

        let self_center_x = self.0.centre.x() as f64;
        let self_center_y = self.0.centre.y() as f64;

        let dx = (point.x().to_float()) - self_center_x;
        let dy = (point.y().to_float()) - self_center_y;

        let new_x = self_center_x + dx * cos_angle - dy * sin_angle;
        let new_y = self_center_y + dx * sin_angle + dy * cos_angle;

        Point::new(
            DatabaseUnitT::from_float(new_x),
            DatabaseUnitT::from_float(new_y),
        )
    }
}
