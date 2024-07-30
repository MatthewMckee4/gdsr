use pyo3::prelude::*;

use crate::path::Path;
use crate::polygon::Polygon;
use crate::reference::Reference;
use crate::text::Text;

mod general;
mod io;

#[pyclass(eq)]
#[derive(Clone, PartialEq, Default)]
pub struct Cell {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub polygons: Vec<Polygon>,
    #[pyo3(get)]
    pub paths: Vec<Path>,
    #[pyo3(get)]
    pub references: Vec<Reference>,
    #[pyo3(get)]
    pub texts: Vec<Text>,
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cell: {} with {} polygons, {} paths, {} references, and {} texts",
            self.name,
            self.polygons.len(),
            self.paths.len(),
            self.references.len(),
            self.texts.len()
        )
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cell({})", self.name)
    }
}
