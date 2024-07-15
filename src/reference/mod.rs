use pyo3::prelude::*;

use crate::cell::Cell;

mod general;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct Reference {
    cell: Cell,
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Reference of {:?}", self.cell)
    }
}

impl std::fmt::Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "R({:?})", self.cell)
    }
}
