use crate::utils::{
    geometry::bounding_box,
    points::{check_vec_not_empty, to_points, to_required_input_points_format},
};
use pyo3::{prelude::*, types::PySequence};

#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Polygon {
    #[pyo3(get)]
    points: Vec<(f64, f64)>,
    #[pyo3(get, set)]
    layer: i32,
    #[pyo3(get, set)]
    data_type: i32,
}

#[pymethods]
impl Polygon {
    #[new]
    #[pyo3(signature = (points, layer=0, data_type=0))]
    pub fn new(
        points: &Bound<'_, PySequence>,
        layer: Option<i32>,
        data_type: Option<i32>,
    ) -> PyResult<Self> {
        let points_list = to_required_input_points_format(points)?;
        let points_vec = to_points(points_list)?;

        let layer = layer.unwrap_or(0);
        let data_type = data_type.unwrap_or(0);
        Ok(Polygon {
            points: points_vec,
            layer,
            data_type,
        })
    }

    #[setter]
    fn set_points(&mut self, points: &Bound<'_, PySequence>) -> PyResult<()> {
        let points_list = to_required_input_points_format(points)?;
        self.points = to_points(points_list)?;
        Ok(())
    }

    #[getter]
    fn get_points(&self) -> PyResult<Vec<(f64, f64)>> {
        check_vec_not_empty(&self.points)?;
        Ok(self.points.clone())
    }

    #[getter]
    fn bounding_box(&self) -> PyResult<((f64, f64), (f64, f64))> {
        Ok(bounding_box(&self.points)?)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "Polygon with {} points, starting at ({}, {}) on layer {}",
            self.points.len(),
            self.points[0].0,
            self.points[0].1,
            self.layer
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        let points_summary = if self.points.len() <= 2 {
            format!("{:?}", self.points)
        } else {
            format!(
                "[{:?}, ..., {:?}]",
                self.points[0],
                self.points[self.points.len() - 1]
            )
        };

        Ok(format!(
            "P({}, {}, {})",
            points_summary, self.layer, self.data_type
        ))
    }
}
