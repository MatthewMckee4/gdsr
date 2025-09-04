use crate::{CoordNum, DatabaseIntegerUnit, Layer, Movable, Transformable};
use crate::{Point, Transformation};

pub mod io;
pub mod presentation;
pub mod utils;

#[derive(Clone, Debug, PartialEq)]
pub struct Text<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub text: String,
    pub origin: Point<DatabaseUnitT>,
    pub layer: Layer,
    pub magnification: f64,
    pub angle: f64,
    pub x_reflection: bool,
    pub vertical_presentation: presentation::VerticalPresentation,
    pub horizontal_presentation: presentation::HorizontalPresentation,
}

impl<DatabaseUnitT: CoordNum> Default for Text<DatabaseUnitT> {
    fn default() -> Self {
        Self {
            text: String::new(),
            origin: Point::new(DatabaseUnitT::zero(), DatabaseUnitT::zero()),
            layer: 0,
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
            vertical_presentation: presentation::VerticalPresentation::default(),
            horizontal_presentation: presentation::HorizontalPresentation::default(),
        }
    }
}

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

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn origin(&self) -> &Point<DatabaseUnitT> {
        &self.origin
    }

    fn set_origin(&mut self, origin: Point<DatabaseUnitT>) {
        self.origin = origin
    }

    pub fn layer(&self) -> Layer {
        self.layer
    }

    pub fn magnification(&self) -> f64 {
        self.magnification
    }

    pub fn angle(&self) -> f64 {
        self.angle
    }

    pub fn x_reflection(&self) -> bool {
        self.x_reflection
    }

    pub fn vertical_presentation(&self) -> &presentation::VerticalPresentation {
        &self.vertical_presentation
    }

    pub fn horizontal_presentation(&self) -> &presentation::HorizontalPresentation {
        &self.horizontal_presentation
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

impl<DatabaseUnitT: CoordNum> Transformable for Text<DatabaseUnitT> {
    fn transform(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();

        if let Some(translation) = &transformation.translation {
            new_self.origin = translation.apply_to_point(new_self.origin())
        }

        if let Some(scale) = &transformation.scale {
            new_self.magnification *= scale.factor()
        }

        if let Some(rotation) = &transformation.rotation {
            if *rotation.centre() == Point::default() {
                new_self.angle += rotation.angle();
            } else {
                todo!()
            }
        }

        new_self
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Text<DatabaseUnitT> {
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self {
        let mut new_self = self.clone();
        new_self.set_origin(Point::new(
            DatabaseUnitT::from_float(target.x().to_float()),
            DatabaseUnitT::from_float(target.y().to_float()),
        ));
        new_self
    }
}
