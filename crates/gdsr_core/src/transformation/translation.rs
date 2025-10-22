use crate::{CoordinateUnit, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct TranslationInner<T: CoordinateUnit = DatabaseIntegerUnit> {
    delta: Point<T>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Translation(TranslationInner<DatabaseIntegerUnit>);

impl Translation {
    pub const fn new(delta: Point<DatabaseIntegerUnit>) -> Self {
        Self(TranslationInner { delta })
    }

    pub fn apply_to_point<T: CoordinateUnit>(&self, point: &Point<T>) -> Point<T> {
        Point::new(
            T::from_float(point.x().to_float_units() + self.0.delta.x() as f64),
            T::from_float(point.y().to_float_units() + self.0.delta.y() as f64),
        )
    }
}
