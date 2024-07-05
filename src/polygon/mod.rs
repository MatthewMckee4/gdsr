use crate::element::Element;
use pyo3::prelude::*;

#[pyclass]
#[derive(Clone)]
pub struct Polygon {}

#[pymethods]
impl Polygon {
    #[new]
    pub fn new() -> Self {
        Polygon {}
    }
}

impl Element for Polygon {
    fn __str__(&self) -> PyResult<String> {
        Ok("Polygon".to_string())
    }
}
