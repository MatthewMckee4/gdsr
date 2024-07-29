use pyo3::prelude::*;

use crate::{
    point::Point,
    traits::{Movable, Rotatable, Scalable},
};

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

impl Movable for Grid {
    fn move_to(&mut self, point: Point) -> &mut Self {
        self.origin = point;
        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        self.origin += vector;
        self
    }
}

impl Rotatable for Grid {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.origin = self.origin.rotate(angle, centre);
        self.angle += angle;
        self
    }
}

impl Scalable for Grid {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        self.origin = self.origin.scale(factor, centre);
        self.spacing_x = self.spacing_x * factor;
        self.spacing_y = self.spacing_y * factor;
        self.magnification *= factor;
        self
    }
}
