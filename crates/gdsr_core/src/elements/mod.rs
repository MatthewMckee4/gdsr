use crate::{CoordNum, DatabaseIntegerUnit, Movable, ToGds, Transformable};

pub mod path;
pub mod polygon;
pub mod reference;
pub mod text;

pub use path::Path;
pub use polygon::Polygon;
pub use reference::Reference;
pub use text::Text;

#[derive(Clone, Debug, PartialEq)]
pub enum Element<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    Path(Path<DatabaseUnitT>),
    Polygon(Polygon<DatabaseUnitT>),
    Reference(Reference<DatabaseUnitT>),
    Text(Text<DatabaseUnitT>),
}

impl<DatabaseUnitT: CoordNum> ToGds for Element<DatabaseUnitT> {
    fn _to_gds(&self, file: &mut std::fs::File, scale: f64) -> std::io::Result<()> {
        match self {
            Element::Path(path) => path._to_gds(file, scale),
            Element::Polygon(polygon) => polygon._to_gds(file, scale),
            Element::Reference(reference) => reference._to_gds(file, scale),
            Element::Text(text) => text._to_gds(file, scale),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Transformable for Element<DatabaseUnitT> {
    fn transform(self, transformation: &crate::Transformation) -> Self {
        match self {
            Element::Path(path) => Element::Path(path.transform(transformation)),
            Element::Polygon(polygon) => Element::Polygon(polygon.transform(transformation)),
            Element::Reference(reference) => {
                Element::Reference(reference.transform(transformation))
            }
            Element::Text(text) => Element::Text(text.transform(transformation)),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Element<DatabaseUnitT> {
    fn move_to(self, target: geo::Point<DatabaseIntegerUnit>) -> Self {
        match self {
            Element::Path(path) => Element::Path(path.move_to(target)),
            Element::Polygon(polygon) => Element::Polygon(polygon.move_to(target)),
            Element::Reference(reference) => Element::Reference(reference.move_to(target)),
            Element::Text(text) => Element::Text(text.move_to(target)),
        }
    }
}
