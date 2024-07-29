use pyo3::prelude::*;

use crate::{
    element::Element,
    grid::Grid,
    point::Point,
    traits::{Movable, Rotatable, Scalable},
};

mod general;
mod io;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct ElementReference {
    #[pyo3(get, set)]
    element: Element,
    #[pyo3(get, set)]
    grid: Grid,
}

impl std::fmt::Display for ElementReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Element Reference of {:?}", self.element)
    }
}

impl std::fmt::Debug for ElementReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ER({:?})", self.element)
    }
}

impl Movable for ElementReference {
    fn move_to(&mut self, point: Point) -> &mut Self {
        self.grid.origin = point;
        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        self.grid.origin += vector;
        self
    }
}

impl Rotatable for ElementReference {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.grid.rotate(angle, centre);
        self
    }
}

impl Scalable for ElementReference {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        self.grid.scale(factor, centre);
        self
    }
}
