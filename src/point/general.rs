use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::exceptions::{PyIndexError, PyZeroDivisionError};

use pyo3::prelude::*;

use crate::utils::geometry::distance_between_points;

use super::iterator::PointIterator;
use super::utils::*;
use super::Point;

#[pymethods]
impl Point {
    #[new]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(
        &self,
        #[pyo3(from_py_with = "py_any_to_point")] other: Point,
    ) -> PyResult<f64> {
        distance_between_points(self, &other)
    }

    pub fn cross(&self, #[pyo3(from_py_with = "py_any_to_point")] other: Point) -> PyResult<f64> {
        Ok(self.x * other.y - self.y * other.x)
    }

    pub fn copy(&self) -> PyResult<Self> {
        Ok(*self)
    }

    #[pyo3(signature = (angle, center=Point { x: 0.0, y: 0.0 }))]
    pub fn rotate(
        &self,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] center: Point,
    ) -> PyResult<Self> {
        let (sin, cos) = angle.to_radians().sin_cos();
        let x = self.x - center.x;
        let y = self.y - center.y;

        let new_x = center.x + x * cos - y * sin;
        let new_y = center.y + x * sin + y * cos;

        let rounded_x = (new_x * 1e10).round() / 1e10;
        let rounded_y = (new_y * 1e10).round() / 1e10;

        Ok(Point {
            x: rounded_x,
            y: rounded_y,
        })
    }

    #[pyo3(signature = (factor, center=Point { x: 0.0, y: 0.0 }))]
    pub fn scale(
        &self,
        factor: f64,
        #[pyo3(from_py_with = "py_any_to_point")] center: Point,
    ) -> PyResult<Self> {
        let x = (self.x - center.x) * factor + center.x;
        let y = (self.y - center.y) * factor + center.y;

        Ok(Point { x, y })
    }

    pub fn __getitem__(&self, index: usize) -> PyResult<f64> {
        match index {
            0 => Ok(self.x),
            1 => Ok(self.y),
            _ => Err(PyIndexError::new_err("Index out of range")),
        }
    }

    pub fn __bool__(&self) -> PyResult<bool> {
        Ok(self.x != 0.0 || self.y != 0.0)
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }

    pub fn __add__(
        &self,
        #[pyo3(from_py_with = "py_any_to_point")] other: Point,
    ) -> PyResult<Self> {
        Ok(Self {
            x: self.x + other.x,
            y: self.y + other.y,
        })
    }

    pub fn __radd__(
        &self,
        #[pyo3(from_py_with = "py_any_to_point")] other: Point,
    ) -> PyResult<Self> {
        self.__add__(other)
    }

    pub fn __sub__(
        &self,
        #[pyo3(from_py_with = "py_any_to_point")] other: Point,
    ) -> PyResult<Self> {
        Ok(Self {
            x: self.x - other.x,
            y: self.y - other.y,
        })
    }

    pub fn __rsub__(
        &self,
        #[pyo3(from_py_with = "py_any_to_point")] other: Point,
    ) -> PyResult<Self> {
        other.__sub__(*self)
    }

    pub fn __mul__(&self, other: f64) -> PyResult<Self> {
        Ok(Self {
            x: self.x * other,
            y: self.y * other,
        })
    }

    pub fn __rmul__(&self, other: f64) -> PyResult<Self> {
        self.__mul__(other)
    }

    pub fn __truediv__(&self, other: f64) -> PyResult<Self> {
        if other == 0.0 {
            return Err(PyZeroDivisionError::new_err("division by zero"));
        }
        Ok(Self {
            x: self.x / other,
            y: self.y / other,
        })
    }

    pub fn __floordiv__(&self, other: f64) -> PyResult<Self> {
        if other == 0.0 {
            return Err(PyZeroDivisionError::new_err("division by zero"));
        }
        Ok(Self {
            x: (self.x / other).floor(),
            y: (self.y / other).floor(),
        })
    }

    pub fn __neg__(&self) -> PyResult<Self> {
        Ok(Self {
            x: -self.x,
            y: -self.y,
        })
    }

    pub fn __hash__(&self) -> PyResult<usize> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish() as usize)
    }

    #[pyo3(signature = (ndigits=None))]
    pub fn __round__(&self, ndigits: Option<i32>) -> PyResult<Self> {
        let factor = match ndigits {
            Some(d) => 10f64.powi(d),
            None => 1.0,
        };
        Ok(Self {
            x: (self.x * factor).round() / factor,
            y: (self.y * factor).round() / factor,
        })
    }

    pub fn __iter__(slf: PyRef<Self>) -> PointIterator {
        PointIterator {
            point: Point { x: slf.x, y: slf.y },
            index: 0,
        }
    }

    pub fn __richcmp__(
        &self,
        #[pyo3(from_py_with = "py_any_to_point")] other: Point,
        op: pyo3::basic::CompareOp,
    ) -> PyResult<bool> {
        match op {
            pyo3::basic::CompareOp::Eq => Ok(self == &other),
            pyo3::basic::CompareOp::Ne => Ok(self != &other),
            pyo3::basic::CompareOp::Lt => Ok(self < &other),
            pyo3::basic::CompareOp::Le => Ok(self <= &other),
            pyo3::basic::CompareOp::Gt => Ok(self > &other),
            pyo3::basic::CompareOp::Ge => Ok(self >= &other),
        }
    }
}
