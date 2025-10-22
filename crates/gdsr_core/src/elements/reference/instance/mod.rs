use std::sync::Arc;

use crate::{
    CoordinateUnit, DatabaseIntegerUnit,
    elements::{Element, Path, Polygon, Reference, Text},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Instance<T: CoordinateUnit = DatabaseIntegerUnit> {
    Cell(String),
    Element(Arc<Box<Element<T>>>),
}

impl<T: CoordinateUnit> Default for Instance<T> {
    fn default() -> Self {
        Self::Cell(String::new())
    }
}

macro_rules! into_instance_impl {
    ($t:ty, $et:expr) => {
        impl<T: CoordinateUnit> From<$t> for Instance<T> {
            fn from(value: $t) -> Self {
                Instance::Element(Arc::new(Box::new($et(value))))
            }
        }

        impl<T: CoordinateUnit> From<$t> for Element<T> {
            fn from(value: $t) -> Self {
                $et(value)
            }
        }
    };
}

into_instance_impl!(Polygon<T>, Element::Polygon);
into_instance_impl!(Path<T>, Element::Path);
into_instance_impl!(Reference<T>, Element::Reference);
into_instance_impl!(Text<T>, Element::Text);

impl<T: CoordinateUnit> From<String> for Instance<T> {
    fn from(value: String) -> Self {
        Self::Cell(value)
    }
}

impl<T: CoordinateUnit> From<&str> for Instance<T> {
    fn from(value: &str) -> Self {
        Self::Cell(value.to_string())
    }
}
