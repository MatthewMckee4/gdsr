use std::ops::DerefMut;

use pyo3::prelude::*;

use crate::{
    element::Element,
    grid::Grid,
    point::{py_any_to_point, Point},
    traits::Movable,
};

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
}
