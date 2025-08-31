mod cell;
mod config;
mod grid;
mod library;
mod path;
mod polygon;
// mod reference;
mod text;
mod traits;
mod transformation;
mod utils;
mod validation;

pub use cell::Cell;
pub use geo::{CoordNum, Point};
pub use grid::Grid;
pub use library::Library;
pub use path::{Path, path_type::PathType};
pub use polygon::Polygon;
// pub use reference::{Reference, instance::Instance};
pub use text::{
    Text,
    presentation::{HorizontalPresentation, VerticalPresentation},
};
pub use traits::{Dimensions, Movable, ToGds, Transformable};
pub use transformation::{Reflection, Rotation, Scale, Transformation, Translation};
pub use validation::input::{
    check_data_type_valid, check_layer_valid, check_points_vec_has_at_least_two_points,
};
