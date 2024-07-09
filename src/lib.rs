use pyo3::prelude::*;

mod utils;

mod array_reference;
mod r#box;
mod cell;
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
use r#box::Box;
use reference::Reference;
use text::Text;

#[pymodule]
fn gdsr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
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
