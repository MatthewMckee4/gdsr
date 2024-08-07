use std::collections::HashMap;

use pyo3::prelude::*;

use crate::cell::Cell;

mod general;
mod io;

#[pyclass]
#[derive(Default)]
pub struct Library {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub cells: HashMap<String, Py<Cell>>,
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library '{}' with {} cells", self.name, self.cells.len())
    }
}

impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library({})", self.name)
    }
}
