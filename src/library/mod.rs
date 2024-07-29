use std::collections::HashMap;

use pyo3::prelude::*;

use crate::cell::Cell;

mod general;
mod io;
mod utils;

#[pyclass]
pub struct Library {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub cells: HashMap<String, Cell>,
}
