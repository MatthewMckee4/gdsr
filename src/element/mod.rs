use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use std::fs::File;

use crate::path::Path;
use crate::point::Point;
use crate::polygon::Polygon;
use crate::reference::Reference;
use crate::text::Text;
use crate::traits::{Movable, Rotatable, Scalable, ToGds};

#[derive(Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Element {
    Path(Path),
    Polygon(Polygon),
    Reference(Box<Reference>),
    Text(Text),
}

impl FromPyObject<'_> for Element {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(element) = ob.extract::<Path>() {
            Ok(Element::Path(element))
        } else if let Ok(element) = ob.extract::<Polygon>() {
            Ok(Element::Polygon(element))
        } else if let Ok(element) = ob.extract::<Reference>() {
            Ok(Element::Reference(Box::new(element)))
        } else if let Ok(element) = ob.extract::<Text>() {
            Ok(Element::Text(element))
        } else {
            Err(PyTypeError::new_err(
                "Element must be a Path, Polygon, Reference, Text, or ElementReference",
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
        match self {
            Element::Path(element) => element._to_gds(file, scale),
            Element::Polygon(element) => element._to_gds(file, scale),
            Element::Reference(element) => element._to_gds(file, scale),
            Element::Text(element) => element._to_gds(file, scale),
        }
    }
}

impl Movable for Element {
    fn move_to(&mut self, point: Point) -> &mut Self {
        match self {
            Element::Path(element) => {
                element.move_to(point);
            }
            Element::Polygon(element) => {
                element.move_to(point);
            }
            Element::Reference(element) => {
                element.move_to(point);
            }
            Element::Text(element) => {
                element.move_to(point);
            }
        }
        self
    }

    fn move_by(&mut self, vector: Point) -> &mut Self {
        match self {
            Element::Path(element) => {
                element.move_by(vector);
            }
            Element::Polygon(element) => {
                element.move_by(vector);
            }
            Element::Reference(element) => {
                element.move_by(vector);
            }
            Element::Text(element) => {
                element.move_by(vector);
            }
        }
        self
    }
}

impl Element {
    pub fn copy(&self) -> Self {
        match self {
            Element::Path(element) => Element::Path(element.copy()),
            Element::Polygon(element) => Element::Polygon(element.copy()),
            Element::Reference(element) => Element::Reference(Box::new(element.copy())),
            Element::Text(element) => Element::Text(element.clone()),
        }
    }
}

impl Rotatable for Element {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        match self {
            Element::Path(element) => {
                element.rotate(angle, centre);
            }
            Element::Polygon(element) => {
                element.rotate(angle, centre);
            }
            Element::Reference(element) => {
                element.rotate(angle, centre);
            }
            Element::Text(element) => {
                element.rotate(angle, centre);
            }
        }
        self
    }
}

impl Scalable for Element {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        match self {
            Element::Path(element) => {
                element.scale(factor, centre);
            }
            Element::Polygon(element) => {
                element.scale(factor, centre);
            }
            Element::Reference(element) => {
                element.scale(factor, centre);
            }
            Element::Text(element) => {
                element.scale(factor, centre);
            }
        }
        self
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Element::Path(element) => write!(f, "{}", element),
            Element::Polygon(element) => write!(f, "{}", element),
            Element::Reference(element) => write!(f, "{}", element),
            Element::Text(element) => write!(f, "{}", element),
        }
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Element::Path(element) => write!(f, "{:?}", element),
            Element::Polygon(element) => write!(f, "{:?}", element),
            Element::Reference(element) => write!(f, "{:?}", element),
            Element::Text(element) => write!(f, "{:?}", element),
        }
    }
}
