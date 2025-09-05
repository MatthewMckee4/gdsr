mod cell;
mod config;
pub(crate) mod elements;
mod grid;
mod library;
mod traits;
mod transformation;
mod utils;

pub use cell::Cell;
pub use elements::{
    Element, HorizontalPresentation, Instance, Path, PathType, Polygon, Reference, Text,
    VerticalPresentation,
};
pub use geo::Point;
pub use grid::Grid;
pub use library::Library;
pub use traits::{Dimensions, Movable, ToGds, Transformable};
pub use transformation::Transformation;

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
        val.round() as Self
    }
}

pub type Layer = u16;
pub type DataType = u16;
