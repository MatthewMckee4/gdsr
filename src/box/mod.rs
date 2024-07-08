use pyo3::prelude::*;

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Box {}

#[pymethods]
impl Box {
    #[new]
    pub fn new() -> Self {
        Box {}
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("Box".to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
