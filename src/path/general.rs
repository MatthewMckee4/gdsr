use std::{f64::consts::PI, ops::DerefMut};

use pyo3::{exceptions::PyNotImplementedError, prelude::*};

use crate::{
    boolean::BooleanOperationResult,
    point::Point,
    polygon::Polygon,
    traits::{Dimensions, LayerDataTypeMatches, Movable, Rotatable, Scalable, Simplifiable},
    utils::{
        geometry::perimeter,
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
    fn setter_points(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>,
    ) -> PyResult<()> {
        check_points_vec_has_at_least_two_points(&points)?;
        self.points = points;
        Ok(())
    }

    fn set_points(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>,
    ) -> PyResult<PyRefMut<'_, Self>> {
        slf.setter_points(points)?;
        Ok(slf)
    }

    #[setter(layer)]
    fn setter_layer(&mut self, layer: i32) -> PyResult<()> {
        check_layer_valid(layer)?;
        self.layer = layer;
        Ok(())
    }

    fn set_layer(mut slf: PyRefMut<'_, Self>, layer: i32) -> PyResult<PyRefMut<'_, Self>> {
        slf.setter_layer(layer)?;
        Ok(slf)
    }

    #[setter(data_type)]
    fn setter_data_type(&mut self, data_type: i32) -> PyResult<()> {
        check_data_type_valid(data_type)?;
        self.data_type = data_type;
        Ok(())
    }

    fn set_data_type(mut slf: PyRefMut<'_, Self>, data_type: i32) -> PyResult<PyRefMut<'_, Self>> {
        slf.setter_data_type(data_type)?;
        Ok(slf)
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

        let first_point = self.points[0];
        let dir_next = (self.points[1] - first_point).normalize();
        let first_normal = dir_next.ortho();

        left_points.push(first_point + first_normal * half_width);
        right_points.push(first_point - first_normal * half_width);

        for window in self.points.windows(3) {
            let (prev, current, next) = (window[0], window[1], window[2]);

            let dir_prev = (current - prev).normalize();
            let dir_next = (next - current).normalize();

            let normal_prev = dir_prev.ortho();
            let normal_next = dir_next.ortho();

            let left_prev = current + normal_prev * half_width;
            let right_prev = current - normal_prev * half_width;

            let left_next = current + normal_next * half_width;
            let right_next = current - normal_next * half_width;

            if let Some(intersection) = line_intersection(left_prev, dir_prev, left_next, dir_next)?
            {
                left_points.push(intersection);
            }

            if let Some(intersection) =
                line_intersection(right_prev, dir_prev, right_next, dir_next)?
            {
                right_points.push(intersection);
            }
        }

        let last_point = self.points[self.points.len() - 1];
        let dir_prev = (last_point - self.points[self.points.len() - 2]).normalize();
        let last_normal = dir_prev.ortho();

        left_points.push(last_point + last_normal * half_width);
        right_points.push(last_point - last_normal * half_width);

        match self.path_type {
            Some(PathType::Round) => {
                let num_points = 16;
                let angle_increment = PI / (num_points) as f64;
                let length_multiplier = 1.0048386;

                for i in 0..num_points {
                    let angle = PI - angle_increment * (i as f64 + 0.5);
                    let cap_point = first_point
                        - dir_next.ortho() * half_width * angle.cos()
                        - dir_next * half_width * angle.sin();
                    left_points.insert(0, cap_point.scale(length_multiplier, first_point));
                }

                for i in 0..num_points {
                    let angle = PI - angle_increment * (i as f64 + 0.5);
                    let cap_point = last_point
                        + dir_prev.ortho() * half_width * angle.cos()
                        + dir_prev * half_width * angle.sin();
                    right_points.push(cap_point.scale(length_multiplier, last_point));
                }
            }
            Some(PathType::Overlap) => {
                let start_point_left = first_point - dir_next * half_width;
                let end_point_left = last_point + dir_prev * half_width;

                let start_point_right = first_point - dir_next * half_width;
                let end_point_right = last_point + dir_prev * half_width;

                let dir_prev = dir_prev.ortho();
                let dir_next = dir_next.ortho();

                left_points.insert(0, start_point_left + dir_next * half_width);
                left_points.push(end_point_left + dir_prev * half_width);

                right_points.insert(0, start_point_right - dir_next * half_width);
                right_points.push(end_point_right - dir_prev * half_width);
            }
            _ => {}
        }

        points.extend(left_points);
        points.extend(right_points.into_iter().rev());

        points.push(points[0]);

        let mut polygon = Polygon::new(
            points,
            layer.unwrap_or(self.layer),
            data_type.unwrap_or(self.data_type),
        )?;

        polygon.simplify();

        Ok(polygon)
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

fn line_intersection(p0: Point, dir0: Point, p1: Point, dir1: Point) -> PyResult<Option<Point>> {
    let denom = dir0.cross(dir1)?;
    if denom.abs() < 1e-8 {
        return Ok(None);
    }
    let delta = p1 - p0;
    let t = delta.cross(dir1)? / denom;
    Ok(Some(p0 + dir0 * t))
}
