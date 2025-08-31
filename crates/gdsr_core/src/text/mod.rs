use log::warn;

use crate::traits::{Dimensions, Movable, Transformable};
use crate::{Point, transformation::Transformation};

pub mod presentation;
pub mod utils;

#[derive(Clone)]
pub struct Text {
    pub text: String,
    pub origin: Point,
    pub layer: i32,
    pub magnification: f64,
    pub angle: f64,
    pub x_reflection: bool,
    pub vertical_presentation: presentation::VerticalPresentation,
    pub horizontal_presentation: presentation::HorizontalPresentation,
}

impl Text {
    pub fn new(
        text: String,
        origin: Point,
        layer: i32,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
        vertical_presentation: presentation::VerticalPresentation,
        horizontal_presentation: presentation::HorizontalPresentation,
    ) -> Self {
        Self {
            text,
            origin,
            layer,
            magnification,
            angle,
            x_reflection,
            vertical_presentation,
            horizontal_presentation,
        }
    }
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
            && (self.origin.x() - other.origin.x()).abs() < f64::EPSILON
            && (self.origin.y() - other.origin.y()).abs() < f64::EPSILON
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

impl Transformable for Text {
    fn transform(&mut self, transformation: &Transformation) -> &mut Self {
        self.origin = transformation.apply_to_point(&self.origin);

        // Apply scale and rotation to text properties
        if let Some(scale) = &transformation.scale {
            self.magnification *= scale.factor;
        }

        if let Some(rotation) = &transformation.rotation {
            self.angle += rotation.angle;
        }

        // Handle reflection
        if transformation.reflection.is_some() {
            self.x_reflection = !self.x_reflection;
        }

        self
    }
}

impl Movable for Text {
    fn move_to(&mut self, target: Point) -> &mut Self {
        self.origin = target;
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

        let lower_left = Point::new(self.origin.x() - half_width, self.origin.y() - half_height);
        let upper_right = Point::new(self.origin.x() + half_width, self.origin.y() + half_height);

        (lower_left, upper_right)
    }
}
