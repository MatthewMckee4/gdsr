use std::{f64::consts::PI, ops::DerefMut};

use log::info;
use pyo3::{exceptions::PyNotImplementedError, prelude::*};

use crate::{
    boolean::BooleanOperationResult,
    point::Point,
    polygon::Polygon,
    traits::{Dimensions, LayerDataTypeMatches, Movable, Rotatable, Scalable},
    utils::{
        geometry::{perimeter, round_to_decimals},
        transformations::{
            py_any_to_boolean_operation_input, py_any_to_point, py_any_to_points_vec,
        },
    },
    validation::input::{
        check_data_type_valid, check_layer_valid, check_points_vec_has_at_least_two_points,
    },
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
        check_points_vec_has_at_least_two_points(&points)?;
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

    #[setter(points)]
    fn setter_points(&mut self, #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>) {
        check_points_vec_has_at_least_two_points(&points).unwrap();
        self.points = points;
    }

    fn set_points(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>,
    ) -> PyRefMut<'_, Self> {
        slf.setter_points(points);
        slf
    }

    #[setter(layer)]
    fn setter_layer(&mut self, layer: i32) -> PyResult<()> {
        check_layer_valid(layer)?;
        self.layer = layer;
        Ok(())
    }

    fn set_layer(mut slf: PyRefMut<'_, Self>, layer: i32) -> PyRefMut<'_, Self> {
        slf.setter_layer(layer).unwrap();
        slf
    }

    #[setter(data_type)]
    fn setter_data_type(&mut self, data_type: i32) -> PyResult<()> {
        check_data_type_valid(data_type)?;
        self.data_type = data_type;
        Ok(())
    }

    fn set_data_type(mut slf: PyRefMut<'_, Self>, data_type: i32) -> PyRefMut<'_, Self> {
        slf.setter_data_type(data_type).unwrap();
        slf
    }

    #[setter(path_type)]
    fn setter_path_type(&mut self, path_type: Option<PathType>) {
        self.path_type = path_type;
    }

    #[pyo3(signature = (path_type=None))]
    fn set_path_type(
        mut slf: PyRefMut<'_, Self>,
        path_type: Option<PathType>,
    ) -> PyRefMut<'_, Self> {
        slf.setter_path_type(path_type);
        slf
    }

    #[setter(width)]
    fn setter_width(&mut self, width: Option<f64>) {
        self.width = width;
    }

    #[pyo3(signature = (width=None))]
    fn set_width(mut slf: PyRefMut<'_, Self>, width: Option<f64>) -> PyRefMut<'_, Self> {
        slf.setter_width(width);
        slf
    }

    #[getter]
    fn length(&self) -> PyResult<f64> {
        perimeter(&self.points)
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

    #[pyo3(signature = (*layer_data_types))]
    pub fn is_on(&self, layer_data_types: Vec<(i32, i32)>) -> bool {
        LayerDataTypeMatches::is_on(self, layer_data_types)
    }

    #[pyo3(signature = (layer=None, data_type=None))]
    pub fn to_polygon(&self, layer: Option<i32>, data_type: Option<i32>) -> PyResult<Polygon> {
        let half_width = self.width.unwrap_or(0.0) / 2.0;
        let mut points: Vec<Point> = Vec::new();

        let mut left_points: Vec<Point> = Vec::new();
        let mut right_points: Vec<Point> = Vec::new();

        for i in 0..self.points.len() {
            let current = self.points[i];

            let dir_prev = if i > 0 {
                (self.points[i] - self.points[i - 1]).normalize()
            } else {
                (self.points[i + 1] - self.points[i]).normalize()
            };

            let dir_next = if i < self.points.len() - 1 {
                (self.points[i + 1] - self.points[i]).normalize()
            } else {
                dir_prev
            };

            let avg_dir = (dir_prev + dir_next).normalize();

            let normal = avg_dir.ortho();

            let left_p = current + normal * half_width;
            let right_p = current - normal * half_width;

            left_points.push(left_p);
            right_points.push(right_p);
        }

        points.extend(left_points);
        points.extend(right_points.into_iter().rev());

        Polygon::new(
            points,
            layer.unwrap_or(self.layer),
            data_type.unwrap_or(self.data_type),
        )
    }

    fn __add__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("or"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __or__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("or"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __and__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("and"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __sub__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("sub"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __xor__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("xor"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

fn segments_intersection(p0: &Point, t0: &Point, p1: &Point, t1: &Point) -> PyResult<(f64, f64)> {
    let den = t0.cross(*t1)?;
    let mut u0 = 0.0;
    let mut u1 = 0.0;
    if den.abs() >= 1e-8 {
        let delta_p = *p1 - *p0;
        u0 = delta_p.cross(*t1)? / den;
        u1 = delta_p.cross(*t0)? / den;
    }
    Ok((u0, u1))
}
