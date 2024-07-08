use pyo3::prelude::*;

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Path {}

#[pymethods]
impl Path {
    #[new]
    pub fn new() -> Self {
        Path {}
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("Path".to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
