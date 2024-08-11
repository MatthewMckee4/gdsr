use std::ops::DerefMut;

use pyo3::prelude::*;

use crate::{
    grid::Grid,
    point::Point,
    traits::{Dimensions, Movable, Rotatable, Scalable},
    utils::transformations::py_any_to_point,
};

use super::{Instance, Reference};

#[pymethods]
impl Reference {
    #[new]
    #[pyo3(signature=(instance, grid=None))]
    pub fn new(instance: Instance, grid: Option<Py<Grid>>) -> Self {
        let grid =
            grid.unwrap_or_else(|| Python::with_gil(|py| Py::new(py, Grid::default()).unwrap()));
        match instance {
            Instance::Cell(cell) => Python::with_gil(|py| Reference {
                instance: Instance::Cell(cell.clone_ref(py)),
                grid,
            }),
            Instance::Element(_) => Reference { instance, grid },
        }
    }

    #[getter]
    fn bounding_box(&self) -> (Point, Point) {
        Dimensions::bounding_box(self)
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
