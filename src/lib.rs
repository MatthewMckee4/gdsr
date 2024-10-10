use config::{get_epsilon, set_epsilon};
use pyo3::prelude::*;

mod utils;

mod boolean;
mod cell;
mod config;
mod element;
mod grid;
mod library;
mod path;
mod point;
mod polygon;
mod reference;
mod text;
mod traits;
mod validation;

use cell::Cell;
use grid::Grid;
use library::Library;
use path::{path_type::PathType, Path};
use point::{Point, PointIterator};
use polygon::Polygon;
use reference::Reference;
use text::{presentation::HorizontalPresentation, presentation::VerticalPresentation, Text};

#[pymodule]
#[pyo3(name = "_gdsr")]
fn gdsr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();
    m.add_class::<Cell>()?;
    m.add_class::<Polygon>()?;
    m.add_class::<Path>()?;
    m.add_class::<Reference>()?;
    m.add_class::<Text>()?;
    m.add_class::<Point>()?;
    m.add_class::<PointIterator>()?;
    m.add_class::<Library>()?;
    m.add_class::<Grid>()?;
    m.add_class::<VerticalPresentation>()?;
    m.add_class::<HorizontalPresentation>()?;
    m.add_class::<PathType>()?;

    let _ = m.add_function(wrap_pyfunction!(set_epsilon, m)?);
    let _ = m.add_function(wrap_pyfunction!(get_epsilon, m)?);

    m.add_function(wrap_pyfunction!(boolean::boolean, m)?)?;

    Ok(())
}
