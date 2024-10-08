use std::ops::DerefMut;

use pyo3::prelude::*;

use super::Grid;
use crate::utils::transformations::py_any_to_point;
use crate::{
    point::Point,
    traits::{Movable, Rotatable, Scalable},
};

#[pymethods]
impl Grid {
    #[new]
    #[pyo3(signature=(
        origin=Point::default(),
        columns=1,
        rows=1,
        spacing_x=Point::default(),
        spacing_y=Point::default(),
        magnification=1.0,
        angle=0.0,
        x_reflection=false
    ))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        #[pyo3(from_py_with = "py_any_to_point")] origin: Point,
        columns: u32,
        rows: u32,
        #[pyo3(from_py_with = "py_any_to_point")] spacing_x: Point,
        #[pyo3(from_py_with = "py_any_to_point")] spacing_y: Point,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
    ) -> Self {
        Grid {
            origin,
            columns,
            rows,
            spacing_x,
            spacing_y,
            magnification,
            angle,
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

    pub fn copy(&self) -> Self {
        self.clone()
    }

    fn move_to(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] point: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_to(slf.deref_mut(), point);
        slf
    }

    fn move_by(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] vector: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_by(slf.deref_mut(), vector);
        slf
    }

    #[pyo3(signature = (angle, centre=Point::default()))]
    fn rotate(
        mut slf: PyRefMut<'_, Self>,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Rotatable::rotate(slf.deref_mut(), angle, centre);
        slf
    }

    #[pyo3(signature = (factor, centre=Point::default()))]
    fn scale(
        mut slf: PyRefMut<'_, Self>,
        factor: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Scalable::scale(slf.deref_mut(), factor, centre);
        slf
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
