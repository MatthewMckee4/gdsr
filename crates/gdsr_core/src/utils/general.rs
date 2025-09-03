use crate::{CoordNum, DatabaseFloatUnit, Point};

pub(crate) fn check_points_vec_not_empty(vec: &[Point]) -> Result<(), String> {
    if vec.is_empty() {
        Err("Points cannot be empty".to_string())
    } else {
        Ok(())
    }
}

pub(crate) fn point_to_database_float<DatabaseUnitT: CoordNum>(
    point: Point<DatabaseUnitT>,
) -> Point<DatabaseFloatUnit> {
    Point::new(point.x().to_float(), point.y().to_float())
}
