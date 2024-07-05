use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::array_reference::ArrayReference;
use crate::node::Node;
use crate::path::Path;
use crate::polygon::Polygon;
use crate::r#box::Box;
use crate::reference::Reference;
use crate::text::Text;

#[pyclass]
#[derive(Clone)]
pub struct Cell {
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

#[pymethods]
impl Cell {
    #[new]
    pub fn new() -> Self {
        Cell {
            array_references: Vec::new(),
            polygons: Vec::new(),
            boxes: Vec::new(),
            nodes: Vec::new(),
            paths: Vec::new(),
            references: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn add(&mut self, element: &PyAny) -> PyResult<()> {
        if let Ok(array_reference) = element.extract::<ArrayReference>() {
            self.array_references.push(array_reference);
        } else if let Ok(polygon) = element.extract::<Polygon>() {
            self.polygons.push(polygon);
        } else if let Ok(r#box) = element.extract::<Box>() {
            self.boxes.push(r#box);
        } else if let Ok(node) = element.extract::<Node>() {
            self.nodes.push(node);
        } else if let Ok(path) = element.extract::<Path>() {
            self.paths.push(path);
        } else if let Ok(reference) = element.extract::<Reference>() {
            self.references.push(reference);
        } else if let Ok(text) = element.extract::<Text>() {
            self.texts.push(text);
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Invalid element type",
            ));
        }
        Ok(())
    }
}
