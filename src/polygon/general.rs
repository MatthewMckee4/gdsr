use std::{f64::consts::PI, ops::DerefMut};

use plotly::{common::Mode, layout::Margin, plot::Plot, Layout, Scatter};

use pyo3::{exceptions::PyNotImplementedError, prelude::*, types::PyTuple};

use crate::{
    boolean::BooleanOperationResult,
    point::{points_are_close, Point},
    traits::{Dimensions, LayerDataTypeMatches, Movable, Rotatable, Scalable, Simplifiable},
    utils::{
        geometry::{area, is_point_inside, is_point_on_edge, perimeter},
        transformations::{
            py_any_to_boolean_operation_input, py_any_to_point, py_any_to_points_vec,
            py_tuple_to_points_vec,
        },
    },
    validation::input::{check_data_type_valid, check_layer_valid},
};

use super::{utils::get_correct_polygon_points_format, Polygon};

#[pymethods]
impl Polygon {
    #[new]
    #[pyo3(signature = (points, layer=0, data_type=0))]
    pub fn new(
        #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>,
        layer: i32,
        data_type: i32,
    ) -> PyResult<Self> {
        check_layer_valid(layer)?;
        check_data_type_valid(data_type)?;

        Ok(Self {
            points: get_correct_polygon_points_format(points),
            layer,
            data_type,
        })
    }

    #[setter(points)]
    fn setter_points(&mut self, #[pyo3(from_py_with = "py_any_to_points_vec")] points: Vec<Point>) {
        self.points = get_correct_polygon_points_format(points);
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

    #[getter]
    fn bounding_box(&self) -> (Point, Point) {
        Dimensions::bounding_box(self)
    }

    #[getter]
    fn area(&self) -> PyResult<f64> {
        area(&self.points)
    }

    #[getter]
    fn perimeter(&self) -> PyResult<f64> {
        perimeter(&self.points)
    }

    fn contains(&self, #[pyo3(from_py_with = "py_any_to_point")] point: Point) -> bool {
        is_point_inside(&point, &self.points)
    }

    #[pyo3(signature = (*points))]
    fn contains_all(&self, points: &Bound<'_, PyTuple>) -> PyResult<bool> {
        let points = py_tuple_to_points_vec(points)?;
        Ok(points.iter().all(|p| is_point_inside(p, &self.points)))
    }

    #[pyo3(signature = (*points))]
    fn contains_any(&self, points: &Bound<'_, PyTuple>) -> PyResult<bool> {
        let points = py_tuple_to_points_vec(points)?;
        Ok(points.iter().any(|p| is_point_inside(p, &self.points)))
    }

    fn on_edge(&self, #[pyo3(from_py_with = "py_any_to_point")] point: Point) -> bool {
        is_point_on_edge(&point, &self.points)
    }

    #[pyo3(signature = (*points))]
    fn on_edge_all(&self, points: &Bound<'_, PyTuple>) -> PyResult<bool> {
        let points = py_tuple_to_points_vec(points)?;
        Ok(points.iter().all(|p| is_point_on_edge(p, &self.points)))
    }

    #[pyo3(signature = (*points))]
    fn on_edge_any(&self, points: &Bound<'_, PyTuple>) -> PyResult<bool> {
        let points = py_tuple_to_points_vec(points)?;
        Ok(points.iter().any(|p| is_point_on_edge(p, &self.points)))
    }

    fn intersects(&self, other: Polygon) -> bool {
        self.points
            .iter()
            .any(|p| is_point_inside(p, &other.points))
            || other
                .points
                .iter()
                .any(|p| is_point_inside(p, &self.points))
    }

    fn visualize(&self) -> PyResult<()> {
        let x: Vec<f64> = self.points.iter().map(|p| p.x).collect();
        let y: Vec<f64> = self.points.iter().map(|p| p.y).collect();

        let trace = Scatter::new(x, y).mode(Mode::Lines).name("Polygon");

        let layout = Layout::new()
            .margin(Margin::new().left(200).right(200).bottom(200).top(200))
            .title(String::from("Polygon Visualization"));

        let mut plot = Plot::new();
        plot.add_trace(trace);
        plot.set_layout(layout);
        plot.show();

        Ok(())
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

    #[staticmethod]
    #[pyo3(signature = (centre, radius, n_sides, rotation=0.0, layer=0, data_type=0))]
    fn regular(
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
        radius: f64,
        n_sides: u32,
        rotation: f64,
        layer: i32,
        data_type: i32,
    ) -> PyResult<Polygon> {
        let mut points = Vec::with_capacity((n_sides + 1) as usize);
        let rotation_rad = rotation.to_radians();

        for i in 0..n_sides {
            let angle = 2.0 * PI * i as f64 / n_sides as f64 + rotation_rad;
            let x = centre.x + radius * angle.cos();
            let y = centre.y + radius * angle.sin();
            points.push(Point { x, y });
        }

        points.push(points[0]);

        Polygon::new(points, layer, data_type)
    }

    #[staticmethod]
    #[pyo3(signature = (centre, horizontal_radius, vertical_radius=None, initial_angle=0.0, final_angle=360.0, n_sides=400, layer=0, data_type=0))]
    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
        horizontal_radius: f64,
        vertical_radius: Option<f64>,
        initial_angle: f64,
        final_angle: f64,
        n_sides: usize,
        layer: i32,
        data_type: i32,
    ) -> Polygon {
        let mut points = Vec::new();

        let final_angle = final_angle.to_radians();
        let initial_angle = initial_angle.to_radians();

        let vertical_radius = vertical_radius.unwrap_or(horizontal_radius);

        let step = (final_angle - initial_angle) / n_sides as f64;

        for i in 0..n_sides {
            let angle = initial_angle + i as f64 * step;
            let x = centre.x + horizontal_radius * angle.cos();
            let y = centre.y + vertical_radius * angle.sin();
            points.push(Point { x, y });
        }
        if final_angle == 2.0 * PI {
            points.push(points[0]);
        } else {
            points.push(centre)
        }

        Polygon {
            points,
            layer,
            data_type,
        }
    }

    pub fn simplify(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        Simplifiable::simplify(slf.deref_mut());
        slf
    }

    fn looks_like(&self, other: &Polygon) -> bool {
        let mut binding = self.clone();
        let self_simplified = binding.simplify();
        let mut binding = other.clone();
        let other_simplified = binding.simplify();

        let self_simplified_points = &self_simplified.points[..self_simplified.points.len() - 1];
        let other_simplified_points = &other_simplified.points[..other_simplified.points.len() - 1];

        if points_are_close(self_simplified_points, other_simplified_points)
            || points_are_close(
                self_simplified_points,
                &other_simplified_points
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        {
            return true;
        }

        for i in 0..self_simplified_points.len() {
            let rotated_self_points = self_simplified_points
                .iter()
                .cycle()
                .skip(i)
                .take(self_simplified_points.len())
                .cloned()
                .collect::<Vec<_>>();

            if points_are_close(&rotated_self_points, other_simplified_points)
                || points_are_close(
                    &rotated_self_points,
                    &other_simplified_points
                        .iter()
                        .rev()
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            {
                return true;
            }
        }

        false
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
