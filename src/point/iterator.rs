use crate::point::Point;
use pyo3::prelude::*;

#[pyclass(subclass)]
pub struct PointIterator {
    pub point: Point,
    pub index: usize,
}

#[pymethods]
impl PointIterator {
    fn __next__(&mut self) -> Option<f64> {
        let result = match self.index {
            0 => Some(self.point.x),
            1 => Some(self.point.y),
            _ => None,
        };
        self.index += 1;
        result
    }
}
