use std::hash::{DefaultHasher, Hash, Hasher};

use pyo3::exceptions::{PyIndexError, PyZeroDivisionError};

use pyo3::prelude::*;

use crate::config::epsilon_is_close;
use crate::utils::{
    geometry::{distance_between_points, round_to_decimals},
    transformations::py_any_to_point,
};

use super::iterator::PointIterator;
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

    pub fn copy(&self) -> Self {
        *self
    }

    #[pyo3(signature = (angle, centre=Point::default()))]
    pub fn rotate(
        &self,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> Self {
        let angle = angle.to_radians();
        let x = centre.x + (self.x - centre.x) * angle.cos() - (self.y - centre.y) * angle.sin();
        let y = centre.y + (self.x - centre.x) * angle.sin() + (self.y - centre.y) * angle.cos();

        Point { x, y }
    }

    #[pyo3(signature = (factor, centre=Point::default()))]
    pub fn scale(
        &self,
        factor: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> Self {
        let x = (self.x - centre.x) * factor + centre.x;
        let y = (self.y - centre.y) * factor + centre.y;

        Point { x, y }
    }

    #[pyo3(signature = (digits=0))]
    pub fn round(&self, digits: u32) -> Self {
        Point {
            x: round_to_decimals(self.x, digits),
            y: round_to_decimals(self.y, digits),
        }
    }

    pub fn angle_to(&self, #[pyo3(from_py_with = "py_any_to_point")] other: Point) -> Option<f64> {
        let dx = other.x - self.x;
        let dy = other.y - self.y;

        if dx == 0.0 && dy == 0.0 {
            return None;
        }

        let angle = dy.atan2(dx).to_degrees();

        if angle < 0.0 { angle + 360.0 } else { angle }.into()
    }

    #[pyo3(signature = (other, rel_tol=1e-6, abs_tol=1e-10))]
    pub fn is_close(
        &self,
        #[pyo3(from_py_with = "py_any_to_point")] other: Point,
        rel_tol: f64,
        abs_tol: f64,
    ) -> bool {
        (self.x - other.x).abs() <= abs_tol + rel_tol * other.x.abs()
            && (self.y - other.y).abs() <= abs_tol + rel_tol * other.y.abs()
    }

    pub fn epsilon_is_close(&self, #[pyo3(from_py_with = "py_any_to_point")] other: Point) -> bool {
        epsilon_is_close(self.x, other.x) && epsilon_is_close(self.y, other.y)
    }

    #[pyo3(signature = (angle, centre=Point::default()))]
    pub fn reflect(
        &self,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> Self {
        let angle_rad = angle.to_radians();

        let translated_x = self.x - centre.x;
        let translated_y = self.y - centre.y;

        let cos_theta = angle_rad.cos();
        let sin_theta = angle_rad.sin();

        let x_new = translated_x * (cos_theta * cos_theta - sin_theta * sin_theta)
            - translated_y * 2.0 * cos_theta * sin_theta;
        let y_new = translated_x * 2.0 * cos_theta * sin_theta
            + translated_y * (sin_theta * sin_theta - cos_theta * cos_theta);

        let x = centre.x + x_new;
        let y = centre.y + y_new;

        Point { x, y }
    }

    pub fn ortho(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub fn normalize(&self) -> Self {
        let norm = self.x.hypot(self.y);
        Self {
            x: self.x / norm,
            y: self.y / norm,
        }
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
        Ok(*self + other)
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
        Ok(*self * other)
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
        let ndigits = match ndigits {
            Some(ndigits) => ndigits as u32,
            None => 0,
        };
        Ok(Self {
            x: round_to_decimals(self.x, ndigits),
            y: round_to_decimals(self.y, ndigits),
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
