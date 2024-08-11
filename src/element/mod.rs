use std::fs::File;

use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    path::Path,
    point::Point,
    polygon::Polygon,
    reference::Reference,
    text::Text,
    traits::{Dimensions, Movable, Rotatable, Scalable, ToGds},
};

#[derive(Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Element {
    Path(Py<Path>),
    Polygon(Py<Polygon>),
    Reference(Py<Reference>),
    Text(Py<Text>),
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| match (self, other) {
            (Element::Path(a), Element::Path(b)) => a.borrow(py).eq(&b.borrow(py)),
            (Element::Polygon(a), Element::Polygon(b)) => a.borrow(py).eq(&b.borrow(py)),
            (Element::Reference(a), Element::Reference(b)) => a.borrow(py).eq(&b.borrow(py)),
            (Element::Text(a), Element::Text(b)) => a.borrow(py).eq(&b.borrow(py)),
            _ => false,
        })
    }
}

impl FromPyObject<'_> for Element {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(element) = ob.extract::<Py<Path>>() {
            Ok(Element::Path(element))
        } else if let Ok(element) = ob.extract::<Py<Polygon>>() {
            Ok(Element::Polygon(element))
        } else if let Ok(element) = ob.extract::<Py<Reference>>() {
            Ok(Element::Reference(element))
        } else if let Ok(element) = ob.extract::<Py<Text>>() {
            Ok(Element::Text(element))
        } else {
            Err(PyTypeError::new_err(
                "Element must be a Path, Polygon, Reference or Text",
            ))
        }
    }
}

impl IntoPy<PyObject> for Element {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Element::Path(path) => path.into_py(py),
            Element::Polygon(polygon) => polygon.into_py(py),
            Element::Reference(reference) => reference.into_py(py),
            Element::Text(text) => text.into_py(py),
        }
    }
}
impl ToGds for Element {
    fn _to_gds(&self, file: File, scale: f64) -> PyResult<File> {
        Python::with_gil(|py| match self {
            Element::Path(element) => element.borrow(py)._to_gds(file, scale),
            Element::Polygon(element) => element.borrow(py)._to_gds(file, scale),
            Element::Reference(element) => element.borrow(py)._to_gds(file, scale),
            Element::Text(element) => element.borrow(py)._to_gds(file, scale),
        })
    }
}

impl Movable for Element {
    fn move_to(&mut self, point: Point) -> &mut Self {
        Python::with_gil(|py| match self {
            Element::Path(element) => {
                element.borrow_mut(py).move_to(point);
            }
            Element::Polygon(element) => {
                element.borrow_mut(py).move_to(point);
            }
            Element::Reference(element) => {
                element.borrow_mut(py).move_to(point);
            }
            Element::Text(element) => {
                element.borrow_mut(py).move_to(point);
            }
        });
        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        Python::with_gil(|py| match self {
            Element::Path(element) => {
                element.borrow_mut(py).move_by(vector);
            }
            Element::Polygon(element) => {
                element.borrow_mut(py).move_by(vector);
            }
            Element::Reference(element) => {
                element.borrow_mut(py).move_by(vector);
            }
            Element::Text(element) => {
                element.borrow_mut(py).move_by(vector);
            }
        });
        self
    }
}

impl Element {
    pub fn copy(&self) -> PyResult<Self> {
        Python::with_gil(|py| {
            Ok(match self {
                Element::Path(element) => Element::Path(Py::new(py, element.borrow(py).clone())?),
                Element::Polygon(element) => {
                    Element::Polygon(Py::new(py, element.borrow(py).clone())?)
                }
                Element::Reference(element) => {
                    Element::Reference(Py::new(py, element.borrow(py).clone())?)
                }
                Element::Text(element) => Element::Text(Py::new(py, element.borrow(py).clone())?),
            })
        })
    }
}

impl Rotatable for Element {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| match self {
            Element::Path(element) => {
                element.borrow_mut(py).rotate(angle, centre);
            }
            Element::Polygon(element) => {
                element.borrow_mut(py).rotate(angle, centre);
            }
            Element::Reference(element) => {
                element.borrow_mut(py).rotate(angle, centre);
            }
            Element::Text(element) => {
                element.borrow_mut(py).rotate(angle, centre);
            }
        });
        self
    }
}

impl Scalable for Element {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        Python::with_gil(|py| match self {
            Element::Path(element) => {
                element.borrow_mut(py).scale(factor, centre);
            }
            Element::Polygon(element) => {
                element.borrow_mut(py).scale(factor, centre);
            }
            Element::Reference(element) => {
                element.borrow_mut(py).scale(factor, centre);
            }
            Element::Text(element) => {
                element.borrow_mut(py).scale(factor, centre);
            }
        });
        self
    }
}

impl Dimensions for Element {
    fn bounding_box(&self) -> (Point, Point) {
        Python::with_gil(|py| match self {
            Element::Path(element) => element.borrow(py).bounding_box(),
            Element::Polygon(element) => element.borrow(py).bounding_box(),
            Element::Reference(element) => element.borrow(py).bounding_box(),
            Element::Text(element) => element.borrow(py).bounding_box(),
        })
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Python::with_gil(|py| match self {
            Element::Path(element) => write!(f, "{}", element.borrow(py).clone()),
            Element::Polygon(element) => write!(f, "{}", element.borrow(py).clone()),
            Element::Reference(element) => write!(f, "{}", element.borrow(py).clone()),
            Element::Text(element) => write!(f, "{}", element.borrow(py).clone()),
        })
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Python::with_gil(|py| match self {
            Element::Path(element) => write!(f, "{:?}", element.borrow(py).clone()),
            Element::Polygon(element) => write!(f, "{:?}", element.borrow(py).clone()),
            Element::Reference(element) => write!(f, "{:?}", element.borrow(py).clone()),
            Element::Text(element) => write!(f, "{:?}", element.borrow(py).clone()),
        })
    }
}
