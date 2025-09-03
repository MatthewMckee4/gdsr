use crate::{CoordNum, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug)]
pub struct TranslationInner<DatabaseUnitT: CoordNum> {
    delta: Point<DatabaseUnitT>,
}

#[derive(Clone, Debug)]
pub struct Translation(TranslationInner<DatabaseIntegerUnit>);

impl Translation {
    pub fn new(delta: Point<DatabaseIntegerUnit>) -> Self {
        Self(TranslationInner { delta })
    }

    pub fn apply_to_point(&self, point: &Point<DatabaseIntegerUnit>) -> Point<DatabaseIntegerUnit> {
        Point::new(point.x() + self.0.delta.x(), point.y() + self.0.delta.y())
    }
}
