use pyo3::prelude::*;

use crate::{element::Element, grid::Grid, point::Point, traits::Movable};

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
    fn move_to(&mut self, point: Point) {
        self.grid.origin = point;
    }

    fn move_by(&mut self, vector: Point) {
        self.grid.origin += vector;
    }
}
