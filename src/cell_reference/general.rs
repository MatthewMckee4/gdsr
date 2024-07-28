use pyo3::prelude::*;

use crate::{
    cell::Cell,
    grid::Grid,
    point::{py_any_to_point, Point},
    traits::Movable,
};

use super::CellReference;

#[pymethods]
impl CellReference {
    #[new]
    #[pyo3(signature=(cell, grid=Grid::default()))]
    pub fn new(cell: Cell, grid: Grid) -> Self {
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

    fn move_to(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_point")] point: Point,
    ) -> PyResult<Self> {
        Movable::move_to(self, point);
        Ok(self.clone())
    }

    fn move_by(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_point")] vector: Point,
    ) -> PyResult<Self> {
        Movable::move_by(self, vector);
        Ok(self.clone())
    }
}
