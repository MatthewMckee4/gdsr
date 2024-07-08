use pyo3::prelude::*;

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Reference {}

#[pymethods]
impl Reference {
    #[new]
    pub fn new() -> Self {
        Reference {}
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("Reference".to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
