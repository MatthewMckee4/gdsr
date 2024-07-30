use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    cell::Cell,
    element::Element,
    grid::Grid,
    point::Point,
    traits::{Movable, Rotatable, Scalable},
};

mod general;
mod io;

#[derive(Clone, PartialEq)]
pub enum ReferenceInstance {
    Cell(Cell),
    Element(Element),
}

impl IntoPy<PyObject> for ReferenceInstance {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            ReferenceInstance::Cell(cell) => cell.into_py(py),
            ReferenceInstance::Element(element) => element.into_py(py),
        }
    }
}

impl FromPyObject<'_> for ReferenceInstance {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(cell) = ob.extract::<Cell>() {
            Ok(ReferenceInstance::Cell(cell))
        } else if let Ok(element) = ob.extract::<Element>() {
            Ok(ReferenceInstance::Element(element))
        } else {
            Err(PyTypeError::new_err(
                "ReferenceInstance must be a Cell or Element",
            ))
        }
    }
}

impl Default for ReferenceInstance {
    fn default() -> Self {
        ReferenceInstance::Cell(Cell::default())
    }
}

impl std::fmt::Display for ReferenceInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReferenceInstance::Cell(cell) => write!(f, "{}", cell),
            ReferenceInstance::Element(element) => write!(f, "{}", element),
        }
    }
}

#[pyclass(eq)]
#[derive(Clone, PartialEq, Default)]
pub struct Reference {
    #[pyo3(get, set)]
    pub instance: ReferenceInstance,
    #[pyo3(get, set)]
    pub grid: Grid,
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.instance {
            ReferenceInstance::Cell(cell) => {
                write!(f, "Reference of {} with {}", cell, self.grid)
            }
            ReferenceInstance::Element(element) => {
                write!(f, "Reference of {} with {}", element, self.grid)
            }
        }
    }
}

impl std::fmt::Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.instance {
            ReferenceInstance::Cell(cell) => write!(f, "Reference({:?})", cell),
            ReferenceInstance::Element(element) => write!(f, "Reference({:?})", element),
        }
    }
}

impl Movable for Reference {
    fn move_to(&mut self, point: Point) -> &mut Self {
        self.grid.move_to(point);
        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        self.grid.move_by(vector);
        self
    }
}

impl Rotatable for Reference {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.grid.rotate(angle, centre);
        self
    }
}

impl Scalable for Reference {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        self.grid.scale(factor, centre);
        self
    }
}
