use std::sync::Arc;

use crate::{
    Cell, CoordNum, DatabaseIntegerUnit,
    elements::{Element, Path, Polygon, Reference, Text},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Instance<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    Cell(Cell<DatabaseUnitT>),
    Element(Arc<Box<Element<DatabaseUnitT>>>),
}

impl<DatabaseUnitT: CoordNum> Default for Instance<DatabaseUnitT> {
    fn default() -> Self {
        Self::Cell(Cell::default())
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

impl<DatabaseUnitT: CoordNum> From<Cell<DatabaseUnitT>> for Instance<DatabaseUnitT> {
    fn from(value: Cell<DatabaseUnitT>) -> Self {
        Self::Cell(value)
    }
}
