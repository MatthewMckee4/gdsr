use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use std::fs::File;

use crate::cell_reference::CellReference;
use crate::element_reference::ElementReference;
use crate::path::Path;
use crate::point::Point;
use crate::polygon::Polygon;
use crate::text::Text;
use crate::traits::{Movable, ToGds};

#[derive(Clone, PartialEq, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Element {
    Path(Path),
    Polygon(Polygon),
    CellReference(CellReference),
    Text(Text),
    ElementReference(Box<ElementReference>),
}

impl FromPyObject<'_> for Element {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(element) = ob.extract::<Path>() {
            Ok(Element::Path(element))
        } else if let Ok(element) = ob.extract::<Polygon>() {
            Ok(Element::Polygon(element))
        } else if let Ok(element) = ob.extract::<CellReference>() {
            Ok(Element::CellReference(element))
        } else if let Ok(element) = ob.extract::<Text>() {
            Ok(Element::Text(element))
        } else if let Ok(element) = ob.extract::<ElementReference>() {
            Ok(Element::ElementReference(Box::new(element)))
        } else {
            Err(PyTypeError::new_err(
                "Element must be a Path, Polygon, CellReference, Text, or ElementReference",
            ))
        }
    }
}

impl IntoPy<PyObject> for Element {
    fn into_py(self, py: Python<'_>) -> PyObject {
        match self {
            Element::Path(element) => element.into_py(py),
            Element::Polygon(element) => element.into_py(py),
            Element::CellReference(element) => element.into_py(py),
            Element::Text(element) => element.into_py(py),
            Element::ElementReference(element) => element.into_py(py),
        }
    }
}

impl ToGds for Element {
    fn _to_gds(&self, file: File, scale: f64) -> PyResult<File> {
        match self {
            Element::Path(element) => element._to_gds(file, scale),
            Element::Polygon(element) => element._to_gds(file, scale),
            Element::CellReference(element) => element._to_gds(file, scale),
            Element::Text(element) => element._to_gds(file, scale),
            Element::ElementReference(element) => element._to_gds(file, scale),
        }
    }
}

impl Movable for Element {
    fn move_to(&mut self, point: Point) {
        match self {
            Element::Path(element) => element.move_to(point),
            Element::Polygon(element) => element.move_to(point),
            Element::CellReference(element) => element.move_to(point),
            Element::Text(element) => element.move_to(point),
            Element::ElementReference(element) => element.move_to(point),
        }
    }

    fn move_by(&mut self, vector: Point) {
        match self {
            Element::Path(element) => element.move_by(vector),
            Element::Polygon(element) => element.move_by(vector),
            Element::CellReference(element) => element.move_by(vector),
            Element::Text(element) => element.move_by(vector),
            Element::ElementReference(element) => element.move_by(vector),
        }
    }
}
