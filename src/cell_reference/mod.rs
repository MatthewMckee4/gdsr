use pyo3::prelude::*;

use crate::{
    cell::Cell,
    grid::Grid,
    point::Point,
    traits::{Movable, Rotatable, Scalable},
};

mod general;
mod io;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct CellReference {
    pub cell: Cell,
    pub grid: Grid,
}

impl std::fmt::Display for CellReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cell Reference of {:?}", self.cell)
    }
}

impl std::fmt::Debug for CellReference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CR({:?})", self.cell)
    }
}

impl Movable for CellReference {
    fn move_to(&mut self, point: Point) -> &mut Self {
        self.grid.move_to(point);
        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        self.grid.move_by(vector);
        self
    }
}

impl Rotatable for CellReference {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.grid.rotate(angle, centre);
        self
    }
}

impl Scalable for CellReference {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        self.grid.scale(factor, centre);
        self
    }
}
