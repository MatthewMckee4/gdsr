use pyo3::prelude::*;

#[pyclass]
pub struct Grid {
    columns: usize,
    rows: usize,
    spacing_x: Point,
}
