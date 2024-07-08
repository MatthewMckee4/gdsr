use pyo3::prelude::*;

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Text {}

#[pymethods]
impl Text {
    #[new]
    pub fn new() -> Self {
        Text {}
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("Text".to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
