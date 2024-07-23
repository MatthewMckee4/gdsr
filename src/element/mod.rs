use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use crate::cell_reference::CellReference;
use crate::element_reference::ElementReference;
use crate::path::Path;
use crate::polygon::Polygon;
use crate::text::Text;

#[derive(Clone, PartialEq, Debug)]
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
