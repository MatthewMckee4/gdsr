use pyo3::prelude::*;

use crate::point::Point;

mod general;
mod io;
pub mod presentation;
mod utils;

#[pyclass(eq)]
#[derive(Clone, PartialEq)]
pub struct Text {
    #[pyo3(get, set)]
    text: String,
    #[pyo3(get)]
    origin: Point,
    #[pyo3(get)]
    layer: i32,
    #[pyo3(get, set)]
    magnification: f64,
    #[pyo3(get, set)]
    angle: f64,
    #[pyo3(get, set)]
    x_reflection: bool,
    #[pyo3(get, set)]
    vertical_presentation: presentation::VerticalPresentation,
    #[pyo3(get, set)]
    horizontal_presentation: presentation::HorizontalPresentation,
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Text '{}' vertical: {:?}, horizontal: {:?} at {:?}",
            self.text, self.vertical_presentation, self.horizontal_presentation, self.origin
        )
    }
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "T({})", self.text)
    }
}
