use pyo3::prelude::*;

mod general;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct Path {}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Path")
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Pa")
    }
}
