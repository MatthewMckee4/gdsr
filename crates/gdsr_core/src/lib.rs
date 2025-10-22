mod cell;
mod config;
pub(crate) mod elements;
mod grid;
mod library;
mod point;
mod traits;
mod transformation;
mod units;
mod utils;

pub use cell::Cell;
pub use elements::{
    Element, HorizontalPresentation, Instance, Path, PathType, Polygon, Reference, Text,
    VerticalPresentation,
};
pub use grid::Grid;
pub use library::Library;
pub use point::Point;
pub use traits::{Dimensions, Movable, ToGds, Transformable};
pub use transformation::Transformation;
pub use units::{CoordinateUnit, DatabaseFloatUnit, DatabaseIntegerUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Layer(u16);

impl Layer {
    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn new(id: u16) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DataType(u16);

impl DataType {
    pub fn value(&self) -> u16 {
        self.0
    }

    pub fn new(id: u16) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AngleInRadians(f64);

impl AngleInRadians {
    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn new(angle: f64) -> Self {
        Self(angle)
    }
}
