use pyo3::prelude::*;

use crate::cell::Cell;

mod general;
mod io;
mod utils;

#[pyclass(subclass)]
pub struct Library {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub cells: Vec<Cell>,
}
