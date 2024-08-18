use pyo3::prelude::*;

use crate::{
    point::Point,
    traits::{Movable, Reflect, Rotatable, Scalable},
};

mod general;

#[pyclass(eq)]
#[derive(Clone)]
pub struct Grid {
    #[pyo3(get)]
    pub origin: Point,
    #[pyo3(get, set)]
    pub columns: u32,
    #[pyo3(get, set)]
    pub rows: u32,
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

impl PartialEq for Grid {
    fn eq(&self, other: &Self) -> bool {
        self.origin.epsilon_is_close(other.origin)
            && self.columns == other.columns
            && self.rows == other.rows
            && self.spacing_x.epsilon_is_close(other.spacing_x)
            && self.spacing_y.epsilon_is_close(other.spacing_y)
            && self.magnification == other.magnification
            && self.angle == other.angle
            && self.x_reflection == other.x_reflection
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Grid at {:?} with {} columns and {} rows, spacing ({:?}, {:?}), magnification {:?}, angle {:?}, x_reflection {}",
            self.origin, self.columns, self.rows, self.spacing_x, self.spacing_y, self.magnification, self.angle, self.x_reflection,
        )
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Grid({:?}, {}, {}, {:?}, {:?}, {:?}, {:?}, {})",
            self.origin,
            self.columns,
            self.rows,
            self.spacing_x,
            self.spacing_y,
            self.magnification,
            self.angle,
            self.x_reflection,
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
        let result = (self.angle + angle) % 360.0;
        let adjusted_result = if result < 0.0 { result + 360.0 } else { result };
        self.angle = adjusted_result;
        self
    }
}

impl Scalable for Grid {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        self.origin = self.origin.scale(factor, centre);
        self.spacing_x = self.spacing_x.scale(factor, centre);
        self.spacing_y = self.spacing_y.scale(factor, centre);
        self.magnification *= factor;
        self
    }
}

impl Reflect for Grid {
    fn reflect(&mut self, angle: f64, centre: Point) -> &mut Self {
        if angle == 0.0 && centre.y == 0.0 {
            self.x_reflection = !self.x_reflection;
        } else {
            self.origin = self.origin.reflect(angle, centre);
            self.spacing_x = self.spacing_x.reflect(angle, centre);
            self.spacing_y = self.spacing_y.reflect(angle, centre);
            self.angle = (self.angle + 2.0 * (angle - self.angle)) % 360.0;
        }
        self
    }
}
