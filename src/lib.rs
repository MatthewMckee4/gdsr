mod array_reference;
mod r#box;
mod cell;
mod element;
mod node;
mod path;
mod polygon;
mod reference;
mod text;

use array_reference::ArrayReference;
use cell::Cell;
use node::Node;
use path::Path;
use polygon::Polygon;
use pyo3::prelude::*;
use r#box::Box;
use reference::Reference;
use text::Text;

#[pymodule]
fn gdsr(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Cell>()?;
    m.add_class::<ArrayReference>()?;
    m.add_class::<Polygon>()?;
    m.add_class::<Box>()?;
    m.add_class::<Node>()?;
    m.add_class::<Path>()?;
    m.add_class::<Reference>()?;
    m.add_class::<Text>()?;
    Ok(())
}
