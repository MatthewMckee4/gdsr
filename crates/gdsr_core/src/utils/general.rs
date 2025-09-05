use crate::{CoordNum, DatabaseFloatUnit, Point};

pub fn point_to_database_float<DatabaseUnitT: CoordNum>(
    point: Point<DatabaseUnitT>,
) -> Point<DatabaseFloatUnit> {
    Point::new(point.x().to_float(), point.y().to_float())
}

pub fn point_to_database_unit<DatabaseUnitT: CoordNum>(
    point: Point<DatabaseFloatUnit>,
) -> Point<DatabaseUnitT> {
    Point::new(
        DatabaseUnitT::from_float(point.x()),
        DatabaseUnitT::from_float(point.y()),
    )
}
