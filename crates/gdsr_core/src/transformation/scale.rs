use crate::{CoordinateUnit, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug, PartialEq)]
struct ScaleInner<T: CoordinateUnit, ScaleT: CoordinateUnit> {
    factor: ScaleT,
    centre: Point<T>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scale(ScaleInner<DatabaseIntegerUnit, f64>);

impl Scale {
    pub const fn new(factor: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        Self(ScaleInner { factor, centre })
    }

    pub const fn factor(&self) -> f64 {
        self.0.factor
    }

    pub const fn centre(&self) -> &Point<DatabaseIntegerUnit> {
        &self.0.centre
    }

    pub fn apply_to_point<T: CoordinateUnit>(&self, point: &Point<T>) -> Point<T> {
        let self_center_x = self.0.centre.x() as f64;
        let self_center_y = self.0.centre.y() as f64;

        let dx = (point.x().to_float_units()) - self_center_x;
        let dy = (point.y().to_float_units()) - self_center_y;

        let new_x = dx.mul_add(self.0.factor, self_center_x);
        let new_y = dy.mul_add(self.0.factor, self_center_y);

        Point::new(T::from_float(new_x), T::from_float(new_y))
    }
}
