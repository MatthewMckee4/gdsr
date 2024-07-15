use pyo3::prelude::*;

mod general;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct Text {}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Text")
    }
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "T")
    }
}
