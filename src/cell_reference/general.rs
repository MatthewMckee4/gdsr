use pyo3::prelude::*;

use crate::{cell::Cell, grid::Grid};

use super::CellReference;

#[pymethods]
impl CellReference {
    #[new]
    #[pyo3(signature=(cell, grid=None))]
    pub fn new(cell: Cell, grid: Option<Grid>) -> Self {
        let grid = grid.unwrap_or(Grid::default());
        CellReference { cell, grid }
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
