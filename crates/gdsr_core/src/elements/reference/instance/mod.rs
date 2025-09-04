use std::sync::Arc;

use crate::{Cell, CoordNum, elements::Element};

#[derive(Clone, Debug, PartialEq)]
pub enum Instance<DatabaseUnitT: CoordNum> {
    Cell(Cell<DatabaseUnitT>),
    Element(Arc<Box<Element<DatabaseUnitT>>>),
}

impl<DatabaseUnitT: CoordNum> Default for Instance<DatabaseUnitT> {
    fn default() -> Self {
        Instance::Cell(Cell::default())
    }
}
