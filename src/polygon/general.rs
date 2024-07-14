use std::fs::File;
use std::io::Write;

use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    point::{py_any_to_point, Point},
    utils::{
        gds_format::write_u16_array_to_file,
        geometry::{area, bounding_box, is_point_inside, is_point_on_edge, perimeter},
    },
};
use log::warn;
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PySequence, PyTuple},
};

use super::{
    utils::{
        check_data_type_valid, check_layer_valid, input_polygon_points_to_correct_format,
        polygon_repr, polygon_str,
    },
    Polygon,
};

impl Polygon {
    pub fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        if self.points.len() > 8190 {
            warn!(
                "{} has more than 8190 points, This may cause errors in the future.",
                self
            );
        }

        let mut polygon_head = [
            4,
            combine_record_and_data_type(GDSRecord::Boundary, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer as u16,
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type as u16,
        ];

        write_u16_array_to_file(&mut polygon_head, &mut file)?;

        let xy_head =
            combine_record_and_data_type(GDSRecord::XY, GDSDataType::FourByteSignedInteger);
        let points_length = self.points.len();
        let mut i0 = 0;

        let mut xy_header_buffer = vec![0u8; 4];
        let mut points_buffer = Vec::with_capacity(8 * 8190);

        while i0 < points_length {
            let i1 = (i0 + 8190).min(points_length);
            let record_length = 4 + 8 * (i1 - i0);

            xy_header_buffer[0..2].copy_from_slice(&(record_length as u16).to_be_bytes());
            xy_header_buffer[2..4].copy_from_slice(&xy_head.to_be_bytes());

            file.write_all(&xy_header_buffer)?;

            points_buffer.clear();
            for point in &self.points[i0..i1] {
                let scaled_x = (point.x * scale).round() as i32;
                let scaled_y = (point.y * scale).round() as i32;

                points_buffer.extend_from_slice(&scaled_x.to_be_bytes());
                points_buffer.extend_from_slice(&scaled_y.to_be_bytes());
            }

            file.write_all(&points_buffer)?;

            i0 = i1;
        }

        let mut polygon_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndEl, GDSDataType::NoData),
        ];

        write_u16_array_to_file(&mut polygon_tail, &mut file)?;

        Ok(file)
    }
}

#[pymethods]
impl Polygon {
    #[new]
    #[pyo3(signature = (points, layer=0, data_type=0))]
    pub fn new(
        #[pyo3(from_py_with = "input_polygon_points_to_correct_format")] points: Vec<Point>,
        layer: Option<i32>,
        data_type: Option<i32>,
    ) -> PyResult<Self> {
        let layer = layer.unwrap_or(0);
        let data_type = data_type.unwrap_or(0);

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
        #[pyo3(from_py_with = "input_polygon_points_to_correct_format")] points: Vec<Point>,
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
        #[pyo3(from_py_with = "input_polygon_points_to_correct_format")] points: Vec<Point>,
    ) -> bool {
        points.iter().all(|p| is_point_inside(p, &self.points))
    }

    #[pyo3(signature = (*points))]
    fn contains_any(
        &self,
        #[pyo3(from_py_with = "input_polygon_points_to_correct_format")] points: Vec<Point>,
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
        #[pyo3(from_py_with = "input_polygon_points_to_correct_format")] points: Vec<Point>,
    ) -> bool {
        points.iter().all(|p| is_point_on_edge(p, &self.points))
    }

    #[pyo3(signature = (*points))]
    fn on_edge_any(
        &self,
        #[pyo3(from_py_with = "input_polygon_points_to_correct_format")] points: Vec<Point>,
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

    #[pyo3(signature = (angle, center=Point { x: 0.0, y: 0.0 }))]
    fn rotate(
        &self,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] center: Point,
    ) -> PyResult<Self> {
        let points = self
            .points
            .iter()
            .map(|p| p.rotate(angle, center))
            .collect::<PyResult<Vec<Point>>>()?;

        Ok(Self {
            points,
            layer: self.layer,
            data_type: self.data_type,
        })
    }

    pub fn __richcmp__(&self, other: Polygon, op: pyo3::basic::CompareOp) -> PyResult<bool> {
        match op {
            pyo3::basic::CompareOp::Eq => Ok(self == &other),
            pyo3::basic::CompareOp::Ne => Ok(self != &other),
            pyo3::basic::CompareOp::Lt => Ok(self < &other),
            pyo3::basic::CompareOp::Le => Ok(self <= &other),
            pyo3::basic::CompareOp::Gt => Ok(self > &other),
            pyo3::basic::CompareOp::Ge => Ok(self >= &other),
        }
    }

    fn copy(&self) -> PyResult<Self> {
        Ok(Self {
            points: self.points.clone(),
            layer: self.layer,
            data_type: self.data_type,
        })
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(polygon_str(self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(polygon_repr(self))
    }
}
