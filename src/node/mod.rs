use pyo3::prelude::*;

mod general;

#[pyclass(subclass, eq)]
#[derive(Clone, PartialEq)]
pub struct Node {}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Node")
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "N")
    }
}
