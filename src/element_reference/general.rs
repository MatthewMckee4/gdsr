use pyo3::prelude::*;

use crate::{element::Element, grid::Grid};

use super::ElementReference;

#[pymethods]
impl ElementReference {
    #[new]
    #[pyo3(signature=(element, grid=None))]
    pub fn new(element: Element, grid: Option<Grid>) -> Self {
        let grid = grid.unwrap_or_default();
        ElementReference { element, grid }
    }

    fn copy(&self) -> PyResult<Self> {
        Ok(self.clone())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
