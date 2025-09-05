use std::sync::Arc;

use crate::{
    CoordNum, DatabaseIntegerUnit,
    elements::{Element, Path, Polygon, Reference, Text},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Instance<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    Cell(String),
    Element(Arc<Box<Element<DatabaseUnitT>>>),
}

impl<DatabaseUnitT: CoordNum> Default for Instance<DatabaseUnitT> {
    fn default() -> Self {
        Self::Cell(String::new())
    }
}

macro_rules! into_instance_impl {
    ($t:ty, $et:expr) => {
        impl<DatabaseUnitT: CoordNum> From<$t> for Instance<DatabaseUnitT> {
            fn from(value: $t) -> Self {
                Instance::Element(Arc::new(Box::new($et(value))))
            }
        }

        impl<DatabaseUnitT: CoordNum> From<$t> for Element<DatabaseUnitT> {
            fn from(value: $t) -> Self {
                $et(value)
            }
        }
    };
}

into_instance_impl!(Polygon<DatabaseUnitT>, Element::Polygon);
into_instance_impl!(Path<DatabaseUnitT>, Element::Path);
into_instance_impl!(Reference<DatabaseUnitT>, Element::Reference);
into_instance_impl!(Text<DatabaseUnitT>, Element::Text);

impl<DatabaseUnitT: CoordNum> From<String> for Instance<DatabaseUnitT> {
    fn from(value: String) -> Self {
        Self::Cell(value)
    }
}

impl<DatabaseUnitT: CoordNum> From<&str> for Instance<DatabaseUnitT> {
    fn from(value: &str) -> Self {
        Self::Cell(value.to_string())
    }
}
