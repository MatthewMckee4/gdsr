use crate::{CoordNum, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug)]
struct ScaleInner<DatabaseUnitT: CoordNum, ScaleT: CoordNum> {
    factor: ScaleT,
    centre: Point<DatabaseUnitT>,
}

#[derive(Clone, Debug)]
pub struct Scale(ScaleInner<DatabaseIntegerUnit, f64>);

impl Scale {
    pub fn new(factor: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        Self(ScaleInner { factor, centre })
    }

    pub fn factor(&self) -> f64 {
        self.0.factor
    }

    pub fn centre(&self) -> &Point<DatabaseIntegerUnit> {
        &self.0.centre
    }

    pub fn apply_to_point(&self, point: &Point<DatabaseIntegerUnit>) -> Point<DatabaseIntegerUnit> {
        let self_center_x = self.0.centre.x() as f64;
        let self_center_y = self.0.centre.y() as f64;

        let dx = (point.x() as f64) - self_center_x;
        let dy = (point.y() as f64) - self_center_y;

        let new_x = self_center_x + dx * self.0.factor;
        let new_y = self_center_y + dy * self.0.factor;

        Point::new(
            new_x.round() as DatabaseIntegerUnit,
            new_y.round() as DatabaseIntegerUnit,
        )
    }
}
