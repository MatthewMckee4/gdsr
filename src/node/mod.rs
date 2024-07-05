use crate::element::Element;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Node {}

#[pymethods]
impl Node {
    #[new]
    pub fn new() -> Self {
        Node {}
    }
}

impl Element for Node {
    fn __str__(&self) -> PyResult<String> {
        Ok("Node".to_string())
    }
}
