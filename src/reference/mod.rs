use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    cell::Cell,
    element::Element,
    grid::Grid,
    point::Point,
    traits::{Dimensions, Movable, Rotatable, Scalable},
};

mod general;
mod io;

#[derive(Clone)]
pub enum ReferenceInstance {
    Cell(Py<Cell>),
    Element(Element),
}

impl ReferenceInstance {
    pub fn __eq__(self, other: ReferenceInstance) -> bool {
        Python::with_gil(|py| match (self, other) {
            (ReferenceInstance::Cell(a), ReferenceInstance::Cell(b)) => {
                a.borrow(py).__eq__(&b.borrow(py))
            }
            (ReferenceInstance::Element(a), ReferenceInstance::Element(b)) => a.__eq__(py, b),
            _ => false,
        })
    }
}

impl PartialEq for ReferenceInstance {
    fn eq(&self, other: &Self) -> bool {
        self.clone().__eq__(other.clone())
    }
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
        if let Ok(cell) = ob.extract::<Py<Cell>>() {
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
        Python::with_gil(|py| ReferenceInstance::Cell(Py::new(py, Cell::default()).unwrap()))
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

impl Movable for ReferenceInstance {
    fn move_to(&mut self, point: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                ReferenceInstance::Cell(cell) => {
                    cell.borrow_mut(py).move_to(point);
                }
                ReferenceInstance::Element(element) => {
                    element.move_to(point);
                }
            };
            self
        })
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                ReferenceInstance::Cell(cell) => {
                    cell.borrow_mut(py).move_by(vector);
                }
                ReferenceInstance::Element(element) => {
                    element.move_by(vector);
                }
            };
            self
        })
    }
}

impl Rotatable for ReferenceInstance {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                ReferenceInstance::Cell(cell) => {
                    cell.borrow_mut(py).rotate(angle, centre);
                }
                ReferenceInstance::Element(element) => {
                    element.rotate(angle, centre);
                }
            };
            self
        })
    }
}

impl Scalable for ReferenceInstance {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            match self {
                ReferenceInstance::Cell(cell) => {
                    cell.borrow_mut(py).scale(factor, centre);
                }
                ReferenceInstance::Element(element) => {
                    element.scale(factor, centre);
                }
            };
            self
        })
    }
}

impl Dimensions for ReferenceInstance {
    fn bounding_box(&self) -> (Point, Point) {
        Python::with_gil(|py| match self {
            ReferenceInstance::Cell(cell) => cell.borrow(py).bounding_box(),
            ReferenceInstance::Element(element) => element.bounding_box(),
        })
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
            ReferenceInstance::Cell(cell) => Python::with_gil(|py| {
                write!(
                    f,
                    "Reference of {} with {}",
                    cell.borrow(py).clone(),
                    self.grid
                )
            }),
            ReferenceInstance::Element(element) => {
                write!(f, "Reference of {} with {}", element, self.grid)
            }
        }
    }
}

impl std::fmt::Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.instance {
            ReferenceInstance::Cell(cell) => {
                Python::with_gil(|py| write!(f, "Reference({:?})", cell.borrow(py).clone()))
            }
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

impl Dimensions for Reference {
    fn bounding_box(&self) -> (Point, Point) {
        let mut min = Point::new(f64::INFINITY, f64::INFINITY);
        let mut max = Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY);

        let corners = vec![
            self.grid.origin,
            self.grid.origin + self.grid.spacing_x * self.grid.columns as f64,
            self.grid.origin + self.grid.spacing_y * self.grid.rows as f64,
            self.grid.origin
                + self.grid.spacing_x * self.grid.columns as f64
                + self.grid.spacing_y * self.grid.rows as f64,
        ];

        for corner in corners {
            let new_element = self
                .instance
                .clone()
                .scale(
                    if self.grid.x_reflection { -1.0 } else { 1.0 },
                    self.grid.origin,
                )
                .scale(self.grid.magnification, self.grid.origin)
                .rotate(self.grid.angle, self.grid.origin)
                .move_by(corner.rotate(self.grid.angle, self.grid.origin).scale(
                    if self.grid.x_reflection { -1.0 } else { 1.0 },
                    self.grid.origin,
                ))
                .clone();

            let (new_element_min, new_element_max) = new_element.bounding_box();

            min.x = min.x.min(new_element_min.x);
            min.y = min.y.min(new_element_min.y);
            max.x = max.x.max(new_element_max.x);
            max.y = max.y.max(new_element_max.y);
        }

        (min, max)
    }
}
