use pyo3::prelude::*;

mod utils;

mod r#box;
mod cell;
mod cell_reference;
mod config;
mod element;
mod element_reference;
mod grid;
mod library;
mod node;
mod path;
mod point;
mod polygon;
mod text;

use cell::Cell;
use cell_reference::CellReference;
use element_reference::ElementReference;
use grid::Grid;
use library::Library;
use node::Node;
use path::Path;
use point::{Point, PointIterator};
use polygon::Polygon;
use r#box::Box;
use text::Text;

#[pymodule]
fn gdsr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    m.add_class::<Cell>()?;
    m.add_class::<Polygon>()?;
    m.add_class::<Box>()?;
    m.add_class::<Node>()?;
    m.add_class::<Path>()?;
    m.add_class::<CellReference>()?;
    m.add_class::<Text>()?;
    m.add_class::<Point>()?;
    m.add_class::<PointIterator>()?;
    m.add_class::<Library>()?;
    m.add_class::<Grid>()?;
    m.add_class::<ElementReference>()?;
    Ok(())
}
