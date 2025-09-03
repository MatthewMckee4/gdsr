use crate::Point;
use crate::{CoordNum, Layer};

pub mod io;
pub mod presentation;
pub mod utils;

#[derive(Clone, Debug, PartialEq)]
pub struct TextInner<DatabaseUnitT: CoordNum> {
    pub text: String,
    pub origin: Point<DatabaseUnitT>,
    pub layer: Layer,
    pub magnification: f64,
    pub angle: f64,
    pub x_reflection: bool,
    pub vertical_presentation: presentation::VerticalPresentation,
    pub horizontal_presentation: presentation::HorizontalPresentation,
}

impl<T: CoordNum> Default for TextInner<T> {
    fn default() -> Self {
        Self {
            text: String::from(""),
            origin: Point::new(T::zero(), T::zero()),
            layer: 0,
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
            vertical_presentation: presentation::VerticalPresentation::default(),
            horizontal_presentation: presentation::HorizontalPresentation::default(),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Text<DatabaseUnitT: CoordNum>(TextInner<DatabaseUnitT>);

impl<DatabaseUnitT: CoordNum> Text<DatabaseUnitT> {
    pub fn new(
        text: String,
        origin: Point<DatabaseUnitT>,
        layer: Layer,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
        vertical_presentation: presentation::VerticalPresentation,
        horizontal_presentation: presentation::HorizontalPresentation,
    ) -> Self {
        Self(TextInner {
            text,
            origin,
            layer,
            magnification,
            angle,
            x_reflection,
            vertical_presentation,
            horizontal_presentation,
        })
    }

    pub fn text(&self) -> &String {
        &self.0.text
    }

    pub fn origin(&self) -> &Point<DatabaseUnitT> {
        &self.0.origin
    }

    pub fn layer(&self) -> Layer {
        self.0.layer
    }

    pub fn magnification(&self) -> f64 {
        self.0.magnification
    }

    pub fn angle(&self) -> f64 {
        self.0.angle
    }

    pub fn x_reflection(&self) -> bool {
        self.0.x_reflection
    }

    pub fn vertical_presentation(&self) -> &presentation::VerticalPresentation {
        &self.0.vertical_presentation
    }

    pub fn horizontal_presentation(&self) -> &presentation::HorizontalPresentation {
        &self.0.horizontal_presentation
    }
}

impl<T: CoordNum> std::fmt::Display for Text<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Text '{}' vertical: {:?}, horizontal: {:?} at {:?}",
            self.text(),
            self.vertical_presentation(),
            self.horizontal_presentation(),
            self.origin()
        )
    }
}
