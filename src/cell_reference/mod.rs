use pyo3::prelude::*;

use crate::{cell::Cell, grid::Grid};

mod general;

#[pyclass(subclass, eq)]
#[derive(Clone, PartialEq)]
pub struct CellReference {
    cell: Cell,
    grid: Grid,
}

impl std::fmt::Display for CellReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cell Reference of {:?}", self.cell)
    }
}

impl std::fmt::Debug for CellReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CR({:?})", self.cell)
    }
}
