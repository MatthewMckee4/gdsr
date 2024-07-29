use pyo3::prelude::*;

use crate::{
    point::Point,
    traits::{Movable, Rotatable, Scalable},
};

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

impl Movable for Text {
    fn move_by(&mut self, delta: Point) -> &mut Self {
        self.origin += delta;
        self
    }

    fn move_to(&mut self, target: Point) -> &mut Self {
        self.origin = target;
        self
    }
}

impl Rotatable for Text {
    fn rotate(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.origin = self.origin.rotate(angle, centre);
        self.angle += angle;
        self
    }
}

impl Scalable for Text {
    fn scale(&mut self, factor: f64, centre: Point) -> &mut Self {
        self.origin = self.origin.scale(factor, centre);
        self.magnification *= factor;
        self
    }
}
