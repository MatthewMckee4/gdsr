use pyo3::prelude::*;

use crate::{element::Element, grid::Grid};

use super::ElementReference;

#[pymethods]
impl ElementReference {
    #[new]
    #[pyo3(signature=(element, grid=Grid::default()))]
    pub fn new(element: Element, grid: Grid) -> Self {
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
