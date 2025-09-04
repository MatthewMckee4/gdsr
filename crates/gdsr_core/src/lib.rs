mod cell;
mod config;
pub mod elements;
mod grid;
mod library;
mod traits;
pub mod transformation;
mod utils;
mod validation;

pub use cell::Cell;
pub use geo::Point;
pub use grid::Grid;
pub use library::Library;

pub use traits::{Dimensions, Movable, ToGds, Transformable};
pub use transformation::Transformation;
pub use validation::input::{
    check_data_type_valid, check_layer_valid, check_points_vec_has_at_least_two_points,
};

pub trait CoordNum: geo::CoordNum {
    fn to_integer(&self) -> DatabaseIntegerUnit;
    fn to_float(&self) -> DatabaseFloatUnit;
    fn from_float(val: DatabaseFloatUnit) -> Self;
}

pub type DatabaseFloatUnit = f64;
pub type DatabaseIntegerUnit = i64;

impl CoordNum for DatabaseFloatUnit {
    fn to_integer(&self) -> DatabaseIntegerUnit {
        self.round() as DatabaseIntegerUnit
    }

    fn to_float(&self) -> DatabaseFloatUnit {
        *self
    }

    fn from_float(val: DatabaseFloatUnit) -> Self {
        val
    }
}
impl CoordNum for DatabaseIntegerUnit {
    fn to_integer(&self) -> DatabaseIntegerUnit {
        *self
    }

    fn to_float(&self) -> DatabaseFloatUnit {
        *self as DatabaseFloatUnit
    }

    fn from_float(val: DatabaseFloatUnit) -> Self {
        val.round() as DatabaseIntegerUnit
    }
}

pub type Layer = u16;
pub type DataType = u16;
