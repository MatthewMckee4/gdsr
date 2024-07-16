use pyo3::prelude::*;

use super::Grid;
use crate::point::Point;

#[pymethods]
impl Grid {
    #[new]
    #[pyo3(signature=(origin=Point { x: 0.0, y: 0.0 }, columns=1, rows=1, spacing_x=Point { x: 0.0, y: 0.0 }, spacing_y=Point { x: 0.0, y: 0.0 }))]
    pub fn new(
        origin: Point,
        columns: usize,
        rows: usize,
        spacing_x: Point,
        spacing_y: Point,
    ) -> Self {
        Grid {
            origin,
            columns,
            rows,
            spacing_x,
            spacing_y,
        }
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
