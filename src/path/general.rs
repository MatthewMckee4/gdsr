use std::ops::DerefMut;

use pyo3::prelude::*;

use crate::{
    point::Point,
    traits::{Dimensions, Movable, Rotatable, Scalable},
    utils::{
        geometry::perimeter,
        transformations::{py_any_to_point, py_any_to_points_vec},
    },
    validation::input::{check_data_type_valid, check_layer_valid},
};

use super::{path_type::PathType, Path};

#[pymethods]
impl Path {
    #[new]
    #[pyo3(signature = (points, layer=0, data_type=0, path_type=None, width=None))]
    pub fn new(
        #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>,
        layer: i32,
        data_type: i32,
        path_type: Option<PathType>,
        width: Option<f64>,
    ) -> PyResult<Self> {
        check_layer_valid(layer)?;
        check_data_type_valid(data_type)?;

        Ok(Self {
            points,
            layer,
            data_type,
            path_type,
            width,
        })
    }

    #[setter]
    fn set_points(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>,
    ) -> PyResult<()> {
        self.points = points;
        Ok(())
    }

    #[setter]
    fn set_layer(&mut self, layer: i32) -> PyResult<()> {
        check_layer_valid(layer)?;
        self.layer = layer;
        Ok(())
    }

    #[setter]
    fn set_data_type(&mut self, data_type: i32) -> PyResult<()> {
        check_data_type_valid(data_type)?;
        self.data_type = data_type;
        Ok(())
    }

    #[getter]
    fn length(&self) -> PyResult<f64> {
        perimeter(&self.points)
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

    #[getter]
    fn bounding_box(&self) -> (Point, Point) {
        Dimensions::bounding_box(self)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
