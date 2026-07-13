mod cell;
mod config;
mod design_rules;
mod diff;
mod elements;
mod error;
mod geometry;
mod grid;
pub(crate) mod io;
mod library;
mod point;
#[cfg(test)]
mod property_tests;
mod stats;
mod traits;
mod transformation;
mod types;
mod units;

pub use cell::Cell;
pub use design_rules::{
    DesignRuleOptions, DesignRuleReport, DesignRuleViolation, DesignRuleViolationKind,
};
pub use diff::{CellDiff, ElementDiff, LibraryDiff, LibraryDiffOptions};
pub use elements::text::{HorizontalPresentation, VerticalPresentation};
pub use elements::{Element, GdsBox, Instance, Node, Path, PathType, Polygon, Reference, Text};
pub use error::GdsError;
pub use grid::Grid;
pub use io::write::svg::cell_to_svg;
pub use io::write::{GdsFileWriter, GdsStreamWriter, GdsWriter};
pub use library::{CellConflictStrategy, DanglingCellReference, Library, LibraryMergeError};
pub use point::Point;
pub use stats::{CellStats, LibraryStats};
pub use traits::{Dimensions, Movable, Transformable};
pub use transformation::{Reflection, Rotation, Scale, Transformation, Translation};
pub use types::{DataType, Degrees, Layer, LayerMapping, Radians};
pub use units::{DEFAULT_FLOAT_UNITS, DEFAULT_INTEGER_UNITS, FloatUnit, IntegerUnit, Unit};
