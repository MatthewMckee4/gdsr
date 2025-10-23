use crate::{Layer, Movable, Point, Transformable};

pub mod io;
pub mod presentation;
pub mod utils;

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub text: String,
    pub origin: Point,
    pub layer: Layer,
    pub magnification: f64,
    pub angle: f64,
    pub x_reflection: bool,
    pub vertical_presentation: presentation::VerticalPresentation,
    pub horizontal_presentation: presentation::HorizontalPresentation,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            origin: Point::integer(0, 0, 1e-9),
            layer: 0,
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
            vertical_presentation: presentation::VerticalPresentation::default(),
            horizontal_presentation: presentation::HorizontalPresentation::default(),
        }
    }
}

impl Text {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        text: String,
        origin: Point,
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

    #[must_use]
    pub const fn text(&self) -> &String {
        &self.text
    }

    #[must_use]
    pub const fn origin(&self) -> &Point {
        &self.origin
    }

    pub const fn set_origin(&mut self, origin: Point) {
        self.origin = origin;
    }

    #[must_use]
    pub const fn layer(&self) -> Layer {
        self.layer
    }

    #[must_use]
    pub const fn magnification(&self) -> f64 {
        self.magnification
    }

    #[must_use]
    pub const fn angle(&self) -> f64 {
        self.angle
    }

    #[must_use]
    pub const fn x_reflection(&self) -> bool {
        self.x_reflection
    }

    #[must_use]
    pub const fn vertical_presentation(&self) -> &presentation::VerticalPresentation {
        &self.vertical_presentation
    }

    #[must_use]
    pub const fn horizontal_presentation(&self) -> &presentation::HorizontalPresentation {
        &self.horizontal_presentation
    }
}

impl std::fmt::Display for Text {
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

impl Transformable for Text {
    fn transform_impl(&self, transformation: &crate::Transformation) -> Self {
        let mut new_self = self.clone();

        if let Some(translation) = &transformation.translation {
            new_self.origin = translation.apply_to_point(new_self.origin());
        }

        if let Some(scale) = &transformation.scale {
            new_self.magnification *= scale.factor();
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

impl Movable for Text {
    fn move_to(&self, target: Point) -> Self {
        let mut new_self = self.clone();
        new_self.set_origin(target);
        new_self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::new(
            "Hello World".to_string(),
            Point::integer(100, 200, 1e-9),
            5,
            2.0,
            45.0,
            true,
            presentation::VerticalPresentation::Top,
            presentation::HorizontalPresentation::Right,
        );

        assert_eq!(text.text(), "Hello World");
        assert_eq!(text.origin(), &Point::integer(100, 200, 1e-9));
        assert_eq!(text.layer(), 5);
        assert_eq!(text.magnification(), 2.0);
        assert_eq!(text.angle(), 45.0);
        assert!(text.x_reflection());
    }

    #[test]
    fn test_text_default() {
        let text = Text::default();

        assert_eq!(text.text(), "");
        assert_eq!(text.origin(), &Point::integer(0, 0, 1e-9));
        assert_eq!(text.layer(), 0);
        assert_eq!(text.magnification(), 1.0);
        assert_eq!(text.angle(), 0.0);
        assert!(!text.x_reflection());
    }

    #[test]
    fn test_text_display() {
        let text = Text::new(
            "Test Text".to_string(),
            Point::integer(10, 20, 1e-9),
            1,
            1.5,
            30.0,
            false,
            presentation::VerticalPresentation::Bottom,
            presentation::HorizontalPresentation::Left,
        );

        let display_str = format!("{text}");
        assert!(display_str.contains("Text 'Test Text'"));
        assert!(display_str.contains("vertical: Bottom"));
        assert!(display_str.contains("horizontal: Left"));
    }

    #[test]
    fn test_text_clone_and_partial_eq() {
        let text1 = Text::new(
            "Clone Test".to_string(),
            Point::integer(5, 10, 1e-9),
            2,
            1.2,
            15.0,
            true,
            presentation::VerticalPresentation::Middle,
            presentation::HorizontalPresentation::Centre,
        );

        let text2 = text1.clone();
        assert_eq!(text1, text2);

        let text3 = Text::new(
            "Different Text".to_string(),
            Point::integer(5, 10, 1e-9),
            2,
            1.2,
            15.0,
            true,
            presentation::VerticalPresentation::Middle,
            presentation::HorizontalPresentation::Centre,
        );
        assert_ne!(text1, text3);
    }
}
