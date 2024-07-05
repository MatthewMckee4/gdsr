use crate::element::Element;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Box {}

#[pymethods]
impl Box {
    #[new]
    pub fn new() -> Self {
        Box {}
    }
}

impl Element for Box {
    fn __str__(&self) -> PyResult<String> {
        Ok("Box".to_string())
    }
}
