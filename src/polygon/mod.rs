use crate::utils::points::to_points;
use pyo3::{prelude::*, types::PyList};

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
        points: &Bound<'_, PyAny>,
        layer: Option<i32>,
        data_type: Option<i32>,
    ) -> PyResult<Self> {
        let points_list = points.downcast::<PyList>()?;
        let points_vec = to_points(points_list)?;

        if points_vec.is_empty() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Points list cannot be empty",
            ));
        }

        let layer = layer.unwrap_or(0);
        let data_type = data_type.unwrap_or(0);
        Ok(Polygon {
            points: points_vec,
            layer,
            data_type,
        })
    }

    #[setter]
    fn set_points(&mut self, points: &Bound<'_, PyAny>) -> PyResult<()> {
        let points_list = points.downcast::<PyList>()?;
        let points_vec = to_points(points_list)?;
        if points_vec.is_empty() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Points list cannot be empty",
            ));
        }

        self.points = points_vec;

        Ok(())
    }

    fn __str__(&self) -> PyResult<String> {
        if self.points.is_empty() {
            Ok(format!("Polygon with 0 points on layer {}", self.layer))
        } else {
            Ok(format!(
                "Polygon with {} points, starting at ({}, {}) on layer {}",
                self.points.len(),
                self.points[0].0,
                self.points[0].1,
                self.layer
            ))
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let points_summary = if self.points.is_empty() {
            "[]".to_string()
        } else if self.points.len() <= 2 {
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
