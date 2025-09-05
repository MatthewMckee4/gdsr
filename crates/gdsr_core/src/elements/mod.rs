use crate::{CoordNum, DatabaseIntegerUnit, Movable, ToGds, Transformable};

pub mod path;
pub mod polygon;
pub mod reference;
pub mod text;

pub use path::{Path, PathType};
pub use polygon::Polygon;
pub use reference::{Instance, Reference};
pub use text::{
    Text,
    presentation::{HorizontalPresentation, VerticalPresentation},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Element<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    Path(Path<DatabaseUnitT>),
    Polygon(Polygon<DatabaseUnitT>),
    Reference(Reference<DatabaseUnitT>),
    Text(Text<DatabaseUnitT>),
}

impl<DatabaseUnitT: CoordNum> ToGds for Element<DatabaseUnitT> {
    fn to_gds_impl(&self, file: &mut std::fs::File, scale: f64) -> std::io::Result<()> {
        match self {
            Self::Path(path) => path.to_gds_impl(file, scale),
            Self::Polygon(polygon) => polygon.to_gds_impl(file, scale),
            Self::Reference(reference) => reference.to_gds_impl(file, scale),
            Self::Text(text) => text.to_gds_impl(file, scale),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Transformable for Element<DatabaseUnitT> {
    fn transform_impl(&self, transformation: &crate::Transformation) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.transform_impl(transformation)),
            Self::Polygon(polygon) => Self::Polygon(polygon.transform_impl(transformation)),
            Self::Reference(reference) => Self::Reference(reference.transform_impl(transformation)),
            Self::Text(text) => Self::Text(text.transform_impl(transformation)),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Element<DatabaseUnitT> {
    fn move_to(&self, target: geo::Point<DatabaseIntegerUnit>) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.move_to(target)),
            Self::Polygon(polygon) => Self::Polygon(polygon.move_to(target)),
            Self::Reference(reference) => Self::Reference(reference.move_to(target)),
            Self::Text(text) => Self::Text(text.move_to(target)),
        }
    }
}
