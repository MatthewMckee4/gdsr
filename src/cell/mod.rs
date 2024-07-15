use pyo3::prelude::*;

use crate::array_reference::ArrayReference;
use crate::node::Node;
use crate::path::Path;
use crate::polygon::Polygon;
use crate::r#box::Box;
use crate::reference::Reference;
use crate::text::Text;

mod general;
mod io;

#[pyclass]
#[derive(Clone, PartialEq)]
pub struct Cell {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get)]
    pub array_references: Vec<ArrayReference>,
    #[pyo3(get)]
    pub polygons: Vec<Polygon>,
    #[pyo3(get)]
    pub boxes: Vec<Box>,
    #[pyo3(get)]
    pub nodes: Vec<Node>,
    #[pyo3(get)]
    pub paths: Vec<Path>,
    #[pyo3(get)]
    pub references: Vec<Reference>,
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
