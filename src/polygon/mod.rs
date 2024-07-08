use pyo3::prelude::*;

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Polygon {}

#[pymethods]
impl Polygon {
    #[new]
    pub fn new() -> Self {
        Polygon {}
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("Polygon".to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
