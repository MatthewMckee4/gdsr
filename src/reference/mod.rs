use pyo3::prelude::*;

use crate::{
    boolean::{
        boolean, BooleanOperationInput, BooleanOperationOperation, BooleanOperationResult,
        ExternalPolygonGroup,
    },
    cell::Cell,
    element::Element,
    grid::Grid,
    point::Point,
    traits::{
        Dimensions, LayerDataTypeMatches, Movable, Reflect, Rotatable, Scalable,
        ToExternalPolygonGroup,
    },
};

mod general;
pub mod instance;
mod io;

pub use instance::Instance;

#[pyclass(eq)]
#[derive(Clone)]
pub struct Reference {
    #[pyo3(get, set)]
    pub instance: Instance,
    #[pyo3(get, set)]
    pub grid: Py<Grid>,
}

impl Reference {
    pub fn boolean(
        &self,
        other: BooleanOperationInput,
        operation: BooleanOperationOperation,
        py: Python,
    ) -> BooleanOperationResult {
        boolean(
            vec![Element::Reference(Py::new(py, self.clone())?)],
            other,
            operation,
            0,
            0,
        )
    }
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.instance {
            Instance::Cell(cell) => Python::with_gil(|py| {
                write!(
                    f,
                    "Reference of {} with {}",
                    cell.borrow(py).clone(),
                    self.grid
                )
            }),
            Instance::Element(element) => {
                write!(f, "Reference of {} with {}", element, self.grid)
            }
        }
    }
}

impl std::fmt::Debug for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.instance {
            Instance::Cell(cell) => {
                Python::with_gil(|py| write!(f, "Reference({:?})", cell.borrow(py).clone()))
            }
            Instance::Element(element) => write!(f, "Reference({:?})", element),
        }
    }
}

impl Default for Reference {
    fn default() -> Self {
        Python::with_gil(|py| Reference {
            instance: Instance::Cell(Py::new(py, Cell::default()).unwrap()),
            grid: Py::new(py, Grid::default()).unwrap(),
        })
    }
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            if self.grid.borrow(py).clone() != other.grid.borrow(py).clone() {
                return false;
            }

            match (&self.instance, &other.instance) {
                (Instance::Cell(cell1), Instance::Cell(cell2)) => {
                    cell1.borrow(py).eq(&cell2.borrow(py))
                }
                (Instance::Element(element1), Instance::Element(element2)) => element1.eq(element2),
                _ => false,
            }
        })
    }
}

impl Movable for Reference {
    fn move_to(&mut self, point: Point) -> &mut Self {
        Python::with_gil(|py| {
            self.grid.borrow_mut(py).move_to(point);
        });
        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        Python::with_gil(|py| {
            self.grid.borrow_mut(py).move_by(vector);
        });
        self
    }
}

impl Rotatable for Reference {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            self.grid.borrow_mut(py).rotate(angle, centre);
        });
        self
    }
}

impl Scalable for Reference {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            self.grid.borrow_mut(py).scale(factor, centre);
        });
        self
    }
}

impl Dimensions for Reference {
    fn bounding_box(&self) -> (Point, Point) {
        let mut min = Point::new(f64::INFINITY, f64::INFINITY);
        let mut max = Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY);

        let grid = Python::with_gil(|py| self.grid.borrow(py).clone());

        let corners = vec![
            grid.origin,
            grid.origin + grid.spacing_x * grid.columns as f64,
            grid.origin + grid.spacing_y * grid.rows as f64,
            grid.origin + grid.spacing_x * grid.columns as f64 + grid.spacing_y * grid.rows as f64,
        ];

        for corner in corners {
            let new_element = self
                .instance
                .clone()
                .scale(if grid.x_reflection { -1.0 } else { 1.0 }, grid.origin)
                .scale(grid.magnification, grid.origin)
                .rotate(grid.angle, grid.origin)
                .move_by(
                    corner
                        .rotate(grid.angle, grid.origin)
                        .scale(if grid.x_reflection { -1.0 } else { 1.0 }, grid.origin),
                )
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

impl Reflect for Reference {
    fn reflect(&mut self, angle: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| {
            self.grid.borrow_mut(py).reflect(angle, centre);
        });
        self
    }
}

impl LayerDataTypeMatches for Reference {
    fn is_on(&self, layer_data_types: Vec<(i32, i32)>) -> bool {
        match &self.instance {
            Instance::Cell(cell) => Python::with_gil(|py| cell.borrow(py).is_on(layer_data_types)),
            Instance::Element(element) => element.is_on(layer_data_types),
        }
    }
}

impl ToExternalPolygonGroup for Reference {
    fn to_external_polygon_group(&self) -> PyResult<ExternalPolygonGroup> {
        Python::with_gil(|py| {
            let mut geometries = Vec::new();
            for element in self.clone().flatten([].to_vec(), None, py) {
                for geometry in element.to_external_polygon_group()? {
                    geometries.push(geometry);
                }
            }

            Ok(ExternalPolygonGroup::new(geometries))
        })
    }
}
