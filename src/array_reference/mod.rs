use pyo3::prelude::*;

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct ArrayReference {}

#[pymethods]
impl ArrayReference {
    #[new]
    pub fn new() -> Self {
        ArrayReference {}
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("ArrayReference".to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
