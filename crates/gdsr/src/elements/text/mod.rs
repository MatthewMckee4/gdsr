use crate::{DataType, Layer, Movable, Point, Transformable};

pub mod io;
pub mod presentation;
pub mod utils;

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub(crate) value: String,
    pub(crate) origin: Point,
    pub(crate) layer: Layer,
    pub(crate) datatype: DataType,
    pub(crate) magnification: f64,
    pub(crate) angle: f64,
    pub(crate) x_reflection: bool,
    pub(crate) vertical_presentation: presentation::VerticalPresentation,
    pub(crate) horizontal_presentation: presentation::HorizontalPresentation,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            value: String::new(),
            origin: Point::integer(0, 0, 1e-9),
            layer: 0,
            datatype: 0,
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
            vertical_presentation: presentation::VerticalPresentation::default(),
            horizontal_presentation: presentation::HorizontalPresentation::default(),
        }
    }
}

impl Text {
    pub fn new(
        text: &str,
        origin: Point,
        layer: Layer,
        datatype: DataType,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
        vertical_presentation: presentation::VerticalPresentation,
        horizontal_presentation: presentation::HorizontalPresentation,
    ) -> Self {
        Self {
            value: text.to_string(),
            origin,
            layer,
            datatype,
            magnification,
            angle,
            x_reflection,
            vertical_presentation,
            horizontal_presentation,
        }
    }

    pub const fn text(&self) -> &String {
        &self.value
    }

    #[must_use]
    pub fn set_text(mut self, text: String) -> Self {
        self.value = text;
        self
    }

    pub const fn origin(&self) -> &Point {
        &self.origin
    }

    #[must_use]
    pub fn set_origin(mut self, origin: Point) -> Self {
        self.origin = origin;
        self
    }

    pub const fn layer(&self) -> Layer {
        self.layer
    }

    #[must_use]
    pub fn set_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    pub const fn magnification(&self) -> f64 {
        self.magnification
    }

    #[must_use]
    pub fn set_magnification(mut self, magnification: f64) -> Self {
        self.magnification = magnification;
        self
    }

    pub const fn angle(&self) -> f64 {
        self.angle
    }

    #[must_use]
    pub fn set_angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    pub const fn x_reflection(&self) -> bool {
        self.x_reflection
    }

    #[must_use]
    pub fn set_x_reflection(mut self, x_reflection: bool) -> Self {
        self.x_reflection = x_reflection;
        self
    }

    pub const fn vertical_presentation(&self) -> &presentation::VerticalPresentation {
        &self.vertical_presentation
    }

    #[must_use]
    pub fn set_vertical_presentation(
        mut self,
        vertical_presentation: presentation::VerticalPresentation,
    ) -> Self {
        self.vertical_presentation = vertical_presentation;
        self
    }

    pub const fn horizontal_presentation(&self) -> &presentation::HorizontalPresentation {
        &self.horizontal_presentation
    }

    #[must_use]
    pub fn set_horizontal_presentation(
        mut self,
        horizontal_presentation: presentation::HorizontalPresentation,
    ) -> Self {
        self.horizontal_presentation = horizontal_presentation;
        self
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Text '{}' vertical: {:?}, horizontal: {:?} at {}",
            self.text(),
            self.vertical_presentation(),
            self.horizontal_presentation(),
            self.origin()
        )
    }
}

impl Transformable for Text {
    fn transform_impl(mut self, transformation: &crate::Transformation) -> Self {
        if let Some(translation) = &transformation.translation {
            self.origin = translation.apply_to_point(self.origin());
        }

        if let Some(scale) = &transformation.scale {
            self.magnification *= scale.factor();
        }

        if let Some(rotation) = &transformation.rotation {
            if *rotation.centre() == Point::default() {
                self.angle += rotation.angle();
            } else {
                self.origin = self
                    .origin
                    .rotate_around_point(rotation.angle(), rotation.centre());
                self.angle += rotation.angle();
            }
        }

        self
    }
}

impl Movable for Text {
    fn move_to(mut self, target: Point) -> Self {
        self.origin = target;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::new(
            "Hello World",
            Point::integer(100, 200, 1e-9),
            5,
            0,
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
            "Test Text",
            Point::integer(10, 20, 1e-9),
            1,
            0,
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
            "Clone Test",
            Point::integer(5, 10, 1e-9),
            2,
            0,
            1.2,
            15.0,
            true,
            presentation::VerticalPresentation::Middle,
            presentation::HorizontalPresentation::Centre,
        );

        let text2 = text1.clone();
        assert_eq!(text1, text2);

        let text3 = Text::new(
            "Different Text",
            Point::integer(5, 10, 1e-9),
            2,
            0,
            1.2,
            15.0,
            true,
            presentation::VerticalPresentation::Middle,
            presentation::HorizontalPresentation::Centre,
        );
        assert_ne!(text1, text3);
    }

    #[test]
    fn test_set_text() {
        let text = Text::default().set_text("New Text".to_string());
        assert_eq!(text.text(), "New Text");
    }

    #[test]
    fn test_set_origin() {
        let new_origin = Point::integer(50, 75, 1e-9);
        let text = Text::default().set_origin(new_origin);
        assert_eq!(text.origin(), &new_origin);
    }

    #[test]
    fn test_set_layer() {
        let text = Text::default().set_layer(10);
        assert_eq!(text.layer(), 10);
    }

    #[test]
    fn test_set_magnification() {
        let text = Text::default().set_magnification(3.5);
        assert_eq!(text.magnification(), 3.5);
    }

    #[test]
    fn test_set_angle() {
        let text = Text::default().set_angle(90.0);
        assert_eq!(text.angle(), 90.0);
    }

    #[test]
    fn test_set_x_reflection() {
        let text = Text::default().set_x_reflection(true);
        assert!(text.x_reflection());
    }

    #[test]
    fn test_set_vertical_presentation() {
        let text =
            Text::default().set_vertical_presentation(presentation::VerticalPresentation::Top);
        assert_eq!(
            text.vertical_presentation(),
            &presentation::VerticalPresentation::Top
        );
    }

    #[test]
    fn test_set_horizontal_presentation() {
        let text = Text::default()
            .set_horizontal_presentation(presentation::HorizontalPresentation::Right);
        assert_eq!(
            text.horizontal_presentation(),
            &presentation::HorizontalPresentation::Right
        );
    }

    #[test]
    fn test_setter_chaining() {
        let text = Text::default()
            .set_text("Chained".to_string())
            .set_origin(Point::integer(100, 200, 1e-9))
            .set_layer(5)
            .set_magnification(2.0)
            .set_angle(45.0)
            .set_x_reflection(true)
            .set_vertical_presentation(presentation::VerticalPresentation::Bottom)
            .set_horizontal_presentation(presentation::HorizontalPresentation::Left);

        assert_eq!(text.text(), "Chained");
        assert_eq!(text.origin(), &Point::integer(100, 200, 1e-9));
        assert_eq!(text.layer(), 5);
        assert_eq!(text.magnification(), 2.0);
        assert_eq!(text.angle(), 45.0);
        assert!(text.x_reflection());
        assert_eq!(
            text.vertical_presentation(),
            &presentation::VerticalPresentation::Bottom
        );
        assert_eq!(
            text.horizontal_presentation(),
            &presentation::HorizontalPresentation::Left
        );
    }
}
