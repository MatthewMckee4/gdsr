use crate::{CoordinateUnit, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug, PartialEq)]
struct ReflectionInner<T: CoordinateUnit, AngleT: CoordinateUnit> {
    angle: AngleT,
    centre: Point<T>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reflection(ReflectionInner<DatabaseIntegerUnit, f64>);

impl Reflection {
    pub const fn new(angle: f64, centre: Point<DatabaseIntegerUnit>) -> Self {
        Self(ReflectionInner { angle, centre })
    }

    pub fn new_horizontal() -> Self {
        Self::new(
            0.0,
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
        )
    }

    pub fn from_line(_point1: &Point<DatabaseIntegerUnit>, _point2: &Point<DatabaseIntegerUnit>) {
        todo!()
    }

    pub fn apply_to_point<T: CoordinateUnit>(&self, point: &Point<T>) -> Point<T> {
        let cos_2angle = (2.0 * self.0.angle).cos();
        let sin_2angle = (2.0 * self.0.angle).sin();

        let self_center_x = self.0.centre.x() as f64;
        let self_center_y = self.0.centre.y() as f64;

        let dx = point.x().to_float_units() - self_center_x;
        let dy = point.y().to_float_units() - self_center_y;

        let new_x = dy.mul_add(sin_2angle, dx.mul_add(cos_2angle, self_center_x));
        let new_y = dy.mul_add(-cos_2angle, dx.mul_add(sin_2angle, self_center_y));

        Point::new(T::from_float(new_x), T::from_float(new_y))
    }
}
