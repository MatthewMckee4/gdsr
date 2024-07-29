use std::ops::DerefMut;

use plotly::layout::Margin;
use plotly::plot::Plot;
use plotly::Layout;
use plotly::{common::Mode, Scatter};

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PySequence, PyTuple},
};

use super::{utils::py_any_to_correct_polygon_points_format, Polygon};
use crate::traits::{Movable, Rotatable, Scalable};
use crate::{
    point::{py_any_to_point, Point},
    utils::geometry::{area, bounding_box, is_point_inside, is_point_on_edge, perimeter},
    validation::input::{check_data_type_valid, check_layer_valid},
};

#[pymethods]
impl Polygon {
    #[new]
    #[pyo3(signature = (points, layer=0, data_type=0))]
    pub fn new(
        #[pyo3(from_py_with = "py_any_to_correct_polygon_points_format")] points: Vec<Point>,
        layer: i32,
        data_type: i32,
    ) -> PyResult<Self> {
        check_layer_valid(layer)?;
        check_data_type_valid(data_type)?;

        Ok(Self {
            points,
            layer,
            data_type,
        })
    }

    #[setter]
    fn set_points(
        &mut self,
        #[pyo3(from_py_with = "py_any_to_correct_polygon_points_format")] points: Vec<Point>,
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
    fn bounding_box(&self) -> PyResult<(Point, Point)> {
        bounding_box(&self.points)
    }

    #[getter]
    fn area(&self) -> PyResult<f64> {
        area(&self.points)
    }

    #[getter]
    fn perimeter(&self) -> PyResult<f64> {
        perimeter(&self.points)
    }

    fn contains(&self, obj: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if let Ok(point) = obj.extract::<Point>() {
                Ok(is_point_inside(&point, &self.points).into_py(py))
            } else if let Ok(seq) = obj.downcast::<PySequence>() {
                let mut results = Vec::new();
                for item in seq.iter()? {
                    let point: Point = item?.extract()?;
                    results.push(is_point_inside(&point, &self.points));
                }
                Ok(PyTuple::new_bound(py, results).into_py(py))
            } else {
                Err(PyValueError::new_err(
                    "Invalid input: expected a Point or a sequence of Points",
                ))
            }
        })
    }

    #[pyo3(signature = (*points))]
    fn contains_all(
        &self,
        #[pyo3(from_py_with = "py_any_to_correct_polygon_points_format")] points: Vec<Point>,
    ) -> bool {
        points.iter().all(|p| is_point_inside(p, &self.points))
    }

    #[pyo3(signature = (*points))]
    fn contains_any(
        &self,
        #[pyo3(from_py_with = "py_any_to_correct_polygon_points_format")] points: Vec<Point>,
    ) -> bool {
        points.iter().any(|p| is_point_inside(p, &self.points))
    }

    fn on_edge(&self, obj: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if let Ok(point) = obj.extract::<Point>() {
                Ok(is_point_on_edge(&point, &self.points).into_py(py))
            } else if let Ok(seq) = obj.downcast::<PySequence>() {
                let mut results = Vec::new();
                for item in seq.iter()? {
                    let point: Point = item?.extract()?;
                    results.push(is_point_on_edge(&point, &self.points));
                }
                Ok(PyTuple::new_bound(py, results).into_py(py))
            } else {
                Err(PyValueError::new_err(
                    "Invalid input: expected a Point or a sequence of Points",
                ))
            }
        })
    }

    #[pyo3(signature = (*points))]
    fn on_edge_all(
        &self,
        #[pyo3(from_py_with = "py_any_to_correct_polygon_points_format")] points: Vec<Point>,
    ) -> bool {
        points.iter().all(|p| is_point_on_edge(p, &self.points))
    }

    #[pyo3(signature = (*points))]
    fn on_edge_any(
        &self,
        #[pyo3(from_py_with = "py_any_to_correct_polygon_points_format")] points: Vec<Point>,
    ) -> bool {
        points.iter().any(|p| is_point_on_edge(p, &self.points))
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

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}
