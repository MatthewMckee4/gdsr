use pyo3::prelude::*;

use crate::{cell::Cell, grid::Grid, point::Point, traits::Movable};

mod general;
mod io;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct CellReference {
    cell: Cell,
    grid: Grid,
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
    fn move_to(&mut self, point: Point) {
        self.grid.origin = point;
    }

    fn move_by(&mut self, vector: Point) {
        self.grid.origin += vector;
    }
}
