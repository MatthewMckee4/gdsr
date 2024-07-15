use pyo3::prelude::*;

use crate::cell::Cell;

mod general;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct ArrayReference {
    cell: Cell,
}

impl std::fmt::Display for ArrayReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ArrayReference of {:?}", self.cell)
    }
}

impl std::fmt::Debug for ArrayReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AR({:?})", self.cell)
    }
}
