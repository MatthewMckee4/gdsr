use std::ops::DerefMut;

use pyo3::{exceptions::PyNotImplementedError, prelude::*};

use crate::{
    boolean::BooleanOperationResult,
    config::FLOATING_POINT_INACCURACY_ROUND_DECIMALS,
    element::Element,
    grid::Grid,
    point::Point,
    traits::{Dimensions, LayerDataTypeMatches, Movable, Reflect, Rotatable, Scalable},
    utils::transformations::{py_any_to_boolean_operation_input, py_any_to_point},
};

use super::{Instance, Reference};

#[pymethods]
impl Reference {
    #[new]
    #[pyo3(signature=(instance, grid=None))]
    pub fn new(instance: Instance, grid: Option<Py<Grid>>) -> Self {
        let grid =
            grid.unwrap_or_else(|| Python::with_gil(|py| Py::new(py, Grid::default()).unwrap()));
        match instance {
            Instance::Cell(cell) => Python::with_gil(|py| Reference {
                instance: Instance::Cell(cell.clone_ref(py)),
                grid,
            }),
            Instance::Element(_) => Reference { instance, grid },
        }
    }

    #[getter]
    fn bounding_box(&self) -> (Point, Point) {
        Dimensions::bounding_box(self)
    }

    pub fn copy(&self) -> Self {
        Python::with_gil(|py| Self {
            instance: match &self.instance {
                Instance::Cell(cell) => {
                    Instance::Cell(Py::new(py, cell.borrow(py).clone()).unwrap())
                }
                Instance::Element(element) => Instance::Element(element.copy()),
            },
            grid: Py::new(py, self.grid.borrow(py).clone()).unwrap(),
        })
    }

    fn move_to(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] point: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_to(slf.deref_mut(), point);
        slf
    }

    fn move_by(
        mut slf: PyRefMut<'_, Self>,
        #[pyo3(from_py_with = "py_any_to_point")] vector: Point,
    ) -> PyRefMut<'_, Self> {
        Movable::move_by(slf.deref_mut(), vector);
        slf
    }

    #[pyo3(signature = (angle, centre=Point::default()))]
    fn rotate(
        mut slf: PyRefMut<'_, Self>,
        angle: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Rotatable::rotate(slf.deref_mut(), angle, centre);
        slf
    }

    #[pyo3(signature = (factor, centre=Point::default()))]
    fn scale(
        mut slf: PyRefMut<'_, Self>,
        factor: f64,
        #[pyo3(from_py_with = "py_any_to_point")] centre: Point,
    ) -> PyRefMut<'_, Self> {
        Scalable::scale(slf.deref_mut(), factor, centre);
        slf
    }

    #[pyo3(signature = (*layer_data_types, depth=None))]
    pub fn flatten(
        &mut self,
        layer_data_types: Vec<(i32, i32)>,
        depth: Option<usize>,
        py: Python,
    ) -> Vec<Element> {
        let depth = depth.unwrap_or(usize::MAX);
        let flatten_all = layer_data_types.is_empty();
        let mut elements: Vec<Element> = Vec::new();
        if depth == 0 {
            return [Element::Reference(Py::new(py, self.copy()).unwrap())].to_vec();
        }
        match &self.instance {
            Instance::Cell(cell) => {
                let flattened_cell_elements =
                    cell.borrow_mut(py)
                        .get_elements(layer_data_types, Some(depth - 1), py);
                for cell_element in flattened_cell_elements {
                    elements.extend(self._get_elements_in_grid(cell_element));
                }
            }
            Instance::Element(element) => match element {
                Element::Path(element) => {
                    if element.borrow(py).is_on(layer_data_types) || flatten_all {
                        elements.extend(
                            self._get_elements_in_grid(Element::Path(element.clone_ref(py))),
                        );
                    };
                }
                Element::Polygon(element) => {
                    if element.borrow(py).is_on(layer_data_types) || flatten_all {
                        elements.extend(
                            self._get_elements_in_grid(Element::Polygon(element.clone_ref(py))),
                        );
                    }
                }
                Element::Text(element) => {
                    if element.borrow(py).is_on(layer_data_types) || flatten_all {
                        elements.extend(
                            self._get_elements_in_grid(Element::Text(element.clone_ref(py))),
                        );
                    }
                }
                Element::Reference(element) => {
                    let flattened_reference_elements = Python::with_gil(|py| {
                        element
                            .borrow_mut(py)
                            .flatten(layer_data_types, Some(depth - 1), py)
                    });

                    let flattened_copied_elements = flattened_reference_elements
                        .iter()
                        .map(|element| element.copy())
                        .collect::<Vec<Element>>();

                    for reference_element in flattened_copied_elements {
                        elements.extend(self._get_elements_in_grid(reference_element).into_iter());
                    }
                }
            },
        }

        elements
    }

    #[pyo3(signature = (*layer_data_types))]
    pub fn is_on(&self, layer_data_types: Vec<(i32, i32)>) -> bool {
        LayerDataTypeMatches::is_on(self, layer_data_types)
    }

    fn __add__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("or"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __or__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("or"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __and__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("and"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __sub__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("sub"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __xor__(&self, obj: &Bound<'_, PyAny>, py: Python) -> PyResult<BooleanOperationResult> {
        match py_any_to_boolean_operation_input(obj) {
            Ok(other) => Ok(self.boolean(other, String::from("xor"), py)),
            Err(_) => Err(PyNotImplementedError::new_err("NotImplemented")),
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

impl Reference {
    pub fn _get_elements_in_grid(&self, element: Element) -> Vec<Element> {
        Python::with_gil(|py| {
            let binding = Py::new(py, self.grid.borrow_mut(py).clone()).unwrap();
            let grid = binding.borrow_mut(py);

            let mut elements: Vec<Element> =
                Vec::with_capacity((grid.columns * grid.rows) as usize);

            for column_index in 0..grid.columns {
                let column_origin = grid.origin + grid.spacing_x * column_index as f64;
                for row_index in 0..grid.rows {
                    let origin = (column_origin + grid.spacing_y * row_index as f64).copy();

                    let mut new_element = element.copy();

                    if grid.x_reflection {
                        new_element.reflect(0.0, Point::new(1.0, 0.0));
                    }

                    new_element.rotate(grid.angle, Point::default());
                    new_element.scale(grid.magnification, Point::default());

                    new_element.move_by(
                        origin
                            .rotate(grid.angle, grid.origin)
                            .round(FLOATING_POINT_INACCURACY_ROUND_DECIMALS),
                    );

                    elements.push(new_element.copy());
                }
            }

            elements
        })
    }
}
