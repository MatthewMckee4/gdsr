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
        Instance::Cell(Cell::default())
    }
}

macro_rules! into_instance_impl {
    ($t:ty, $et:expr) => {
        impl<DatabaseUnitT: CoordNum> Into<Instance<DatabaseUnitT>> for $t {
            fn into(self) -> Instance<DatabaseUnitT> {
                Instance::Element(Arc::new(Box::new($et(self))))
            }
        }

        impl<DatabaseUnitT: CoordNum> Into<Element<DatabaseUnitT>> for $t {
            fn into(self) -> Element<DatabaseUnitT> {
                $et(self)
            }
        }
    };
}

into_instance_impl!(Polygon<DatabaseUnitT>, Element::Polygon);
into_instance_impl!(Path<DatabaseUnitT>, Element::Path);
into_instance_impl!(Reference<DatabaseUnitT>, Element::Reference);
into_instance_impl!(Text<DatabaseUnitT>, Element::Text);
