use crate::{CoordNum, DatabaseIntegerUnit, Point};

#[derive(Clone, Debug)]
pub struct TranslationInner<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    delta: Point<DatabaseUnitT>,
}

#[derive(Clone, Debug)]
pub struct Translation(TranslationInner<DatabaseIntegerUnit>);

impl Translation {
    pub fn new(delta: Point<DatabaseIntegerUnit>) -> Self {
        Self(TranslationInner { delta })
    }

    pub fn apply_to_point<DatabaseUnitT: CoordNum>(
        &self,
        point: &Point<DatabaseUnitT>,
    ) -> Point<DatabaseUnitT> {
        Point::new(
            DatabaseUnitT::from_float(point.x().to_float() + self.0.delta.x() as f64),
            DatabaseUnitT::from_float(point.y().to_float() + self.0.delta.y() as f64),
        )
    }
}
