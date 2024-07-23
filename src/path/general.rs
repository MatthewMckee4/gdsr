use pyo3::prelude::*;

use crate::utils::geometry::perimeter;
use crate::validation::input::{check_data_type_valid, input_points_like_to_points_vec};
use crate::{point::Point, validation::input::check_layer_valid};

use super::{path_type::PathType, Path};

#[pymethods]
impl Path {
    #[new]
    #[pyo3(signature = (points, layer=0, data_type=0, path_type=None, width=None))]
    pub fn new(
        #[pyo3(from_py_with = "input_points_like_to_points_vec")] points: Vec<Point>,
        layer: i32,
        data_type: i32,
        path_type: Option<PathType>,
        width: Option<i32>,
    ) -> Self {
        Path {
            points,
            layer,
            data_type,
            path_type,
            width,
        }
    }

    #[setter]
    fn set_points(
        &mut self,
        #[pyo3(from_py_with = "input_points_like_to_points_vec")] points: Vec<Point>,
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
