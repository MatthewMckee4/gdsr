use log::warn;
use pyo3::prelude::*;

use crate::point::Point;
use crate::traits::{Dimensions, Movable, Reflect, Rotatable, Scalable};

mod general;
mod io;
pub mod presentation;
pub mod utils;

#[pyclass(eq)]
#[derive(Clone)]
pub struct Text {
    #[pyo3(get, set)]
    pub text: String,
    #[pyo3(get)]
    pub origin: Point,
    #[pyo3(get)]
    pub layer: i32,
    #[pyo3(get, set)]
    pub magnification: f64,
    #[pyo3(get, set)]
    pub angle: f64,
    #[pyo3(get, set)]
    pub x_reflection: bool,
    #[pyo3(get, set)]
    pub vertical_presentation: presentation::VerticalPresentation,
    #[pyo3(get, set)]
    pub horizontal_presentation: presentation::HorizontalPresentation,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::from(""),
            origin: Point::default(),
            layer: 0,
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
            vertical_presentation: presentation::VerticalPresentation::default(),
            horizontal_presentation: presentation::HorizontalPresentation::default(),
        }
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.origin.epsilon_is_close(other.origin)
            && self.layer == other.layer
            && self.magnification == other.magnification
            && self.angle == other.angle
            && self.x_reflection == other.x_reflection
            && self.vertical_presentation == other.vertical_presentation
            && self.horizontal_presentation == other.horizontal_presentation
    }
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
        write!(
            f,
            "Text({}, {:?}, {:?}, {:?}, {}, {}, {:?}, {:?})",
            self.text,
            self.origin,
            self.layer,
            self.magnification,
            self.angle,
            self.x_reflection,
            self.vertical_presentation,
            self.horizontal_presentation
        )
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

impl Dimensions for Text {
    fn bounding_box(&self) -> (Point, Point) {
        warn!("Bounding box of text is not implemented yet. Returning a box around the text.");
        let width = self.text.len() as f64 * self.magnification;
        let height = self.magnification;
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let lower_left = self.origin - Point::new(half_width, half_height);
        let upper_right = self.origin + Point::new(half_width, half_height);

        (lower_left, upper_right)
    }
}

impl Reflect for Text {
    fn reflect(&mut self, angle: f64, centre: Point) -> &mut Self {
        self.origin = self.origin.reflect(angle, centre);
        self.angle = (self.angle + 2.0 * (angle - self.angle)) % 360.0;
        self
    }
}
