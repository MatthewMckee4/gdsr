use pyo3::prelude::*;

use crate::cell_reference::CellReference;
use crate::element_reference::ElementReference;
use crate::path::Path;
use crate::polygon::Polygon;
use crate::text::Text;

mod general;
mod io;

#[pyclass(subclass)]
#[derive(Clone, PartialEq)]
pub struct Cell {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub polygons: Vec<Polygon>,
    #[pyo3(get)]
    pub paths: Vec<Path>,
    #[pyo3(get)]
    pub cell_references: Vec<CellReference>,
    #[pyo3(get)]
    pub element_references: Vec<ElementReference>,
    #[pyo3(get)]
    pub texts: Vec<Text>,
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cell: {}", self.name)
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
