use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    cell::Cell,
    element::Element,
    point::Point,
    traits::{Dimensions, Movable, Rotatable, Scalable},
};

#[derive(Clone)]
pub enum Instance {
    Cell(Py<Cell>),
    Element(Element),
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| match (self, other) {
            (Instance::Cell(a), Instance::Cell(b)) => a.borrow(py).eq(&b.borrow(py)),
            (Instance::Element(a), Instance::Element(b)) => a.clone().eq(&b.clone()),
            _ => false,
        })
    }
}

impl IntoPy<PyObject> for Instance {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Instance::Cell(cell) => cell.into_py(py),
            Instance::Element(element) => element.into_py(py),
        }
    }
}

impl FromPyObject<'_> for Instance {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(cell) = ob.extract::<Py<Cell>>() {
            Ok(Instance::Cell(cell))
        } else if let Ok(element) = ob.extract::<Element>() {
            Ok(Instance::Element(element))
        } else {
            Err(PyTypeError::new_err(
                "ReferenceInstance must be a Cell or Element",
            ))
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Python::with_gil(|py| Instance::Cell(Py::new(py, Cell::default()).unwrap()))
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Instance::Cell(cell) => write!(f, "{}", cell),
            Instance::Element(element) => write!(f, "{}", element),
        }
    }
}

impl Movable for Instance {
    fn move_to(&mut self, point: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                Instance::Cell(cell) => {
                    cell.borrow_mut(py).move_to(point);
                }
                Instance::Element(element) => {
                    element.move_to(point);
                }
            };
            self
        })
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                Instance::Cell(cell) => {
                    cell.borrow_mut(py).move_by(vector);
                }
                Instance::Element(element) => {
                    element.move_by(vector);
                }
            };
            self
        })
    }
}

impl Rotatable for Instance {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                Instance::Cell(cell) => {
                    cell.borrow_mut(py).rotate(angle, centre);
                }
                Instance::Element(element) => {
                    element.rotate(angle, centre);
                }
            };
            self
        })
    }
}

impl Scalable for Instance {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                Instance::Cell(cell) => {
                    cell.borrow_mut(py).scale(factor, centre);
                }
                Instance::Element(element) => {
                    element.scale(factor, centre);
                }
            };
            self
        })
    }
}

impl Dimensions for Instance {
    fn bounding_box(&self) -> (Point, Point) {
        Python::with_gil(|py| match self {
            Instance::Cell(cell) => cell.borrow(py).bounding_box(),
            Instance::Element(element) => element.bounding_box(),
        })
    }
}

impl Instance {
    pub fn copy(&self) -> Self {
        Python::with_gil(|py| match self {
            Instance::Cell(cell) => Instance::Cell(Py::new(py, cell.borrow(py).clone()).unwrap()),
            Instance::Element(element) => Instance::Element(element.copy()),
        })
    }
}
