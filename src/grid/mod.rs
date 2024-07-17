use pyo3::prelude::*;

use crate::point::Point;

mod general;

#[pyclass(subclass)]
#[derive(Clone, PartialEq)]
pub struct Grid {
    #[pyo3(get, set)]
    origin: Point,
    #[pyo3(get, set)]
    columns: usize,
    #[pyo3(get, set)]
    rows: usize,
    #[pyo3(get, set)]
    spacing_x: Point,
    #[pyo3(get, set)]
    spacing_y: Point,
}

impl Default for Grid {
    fn default() -> Self {
        Grid {
            origin: Point { x: 0.0, y: 0.0 },
            columns: 1,
            rows: 1,
            spacing_x: Point { x: 0.0, y: 0.0 },
            spacing_y: Point { x: 0.0, y: 0.0 },
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
