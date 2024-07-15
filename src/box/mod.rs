use pyo3::prelude::*;

mod general;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct Box {}

impl std::fmt::Display for Box {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Box")
    }
}

impl std::fmt::Debug for Box {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "B")
    }
}
