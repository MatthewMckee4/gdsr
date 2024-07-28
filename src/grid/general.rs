use pyo3::prelude::*;

use super::Grid;
use crate::point::{py_any_to_point, Point};

#[pymethods]
impl Grid {
    #[new]
    #[pyo3(signature=(
        origin=Point::default(),
        columns=1,
        rows=1,
        spacing_x=Point::default(),
        spacing_y=Point::default(),
        angle=0.0,
        magnification=1.0,
        x_reflection=false
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        #[pyo3(from_py_with = "py_any_to_point")] origin: Point,
        columns: usize,
        rows: usize,
        #[pyo3(from_py_with = "py_any_to_point")] spacing_x: Point,
        #[pyo3(from_py_with = "py_any_to_point")] spacing_y: Point,
        angle: f64,
        magnification: f64,
        x_reflection: bool,
    ) -> Self {
        Grid {
            origin,
            columns,
            rows,
            spacing_x,
            spacing_y,
            angle,
            magnification,
            x_reflection,
        }
    }

    #[setter]
    pub fn set_origin(&mut self, #[pyo3(from_py_with = "py_any_to_point")] origin: Point) {
        self.origin = origin;
    }

    #[setter]
    pub fn set_spacing_x(&mut self, #[pyo3(from_py_with = "py_any_to_point")] spacing_x: Point) {
        self.spacing_x = spacing_x;
    }

    #[setter]
    pub fn set_spacing_y(&mut self, #[pyo3(from_py_with = "py_any_to_point")] spacing_y: Point) {
        self.spacing_y = spacing_y;
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
