use pyo3::prelude::*;

use crate::point::Point;

mod general;

#[pyclass]
#[derive(Clone, PartialEq)]
pub struct Grid {
    #[pyo3(get)]
    pub origin: Point,
    #[pyo3(get, set)]
    pub columns: usize,
    #[pyo3(get, set)]
    pub rows: usize,
    #[pyo3(get)]
    pub spacing_x: Point,
    #[pyo3(get)]
    pub spacing_y: Point,
    #[pyo3(get, set)]
    pub magnification: f64,
    #[pyo3(get, set)]
    pub angle: f64,
    #[pyo3(get, set)]
    pub x_reflection: bool,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            origin: Point::default(),
            columns: 1,
            rows: 1,
            spacing_x: Point::default(),
            spacing_y: Point::default(),
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
        }
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Grid at ({}, {}) with {} columns and {} rows, spacing ({}, {})",
            self.origin.x, self.origin.y, self.columns, self.rows, self.spacing_x, self.spacing_y
        )
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "G({}, {}, {}, {}, {})",
            self.origin, self.columns, self.rows, self.spacing_x, self.spacing_y
        )
    }
}
