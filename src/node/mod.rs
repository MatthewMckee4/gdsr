use pyo3::prelude::*;

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Node {}

#[pymethods]
impl Node {
    #[new]
    pub fn new() -> Self {
        Node {}
    }

    fn __str__(&self) -> PyResult<String> {
        Ok("Node".to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}
