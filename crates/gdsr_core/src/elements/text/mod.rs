use crate::{CoordNum, DatabaseIntegerUnit, Layer, Movable, Point, Transformable, Transformation};

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
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
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

    pub const fn text(&self) -> &String {
        &self.text
    }

    pub const fn origin(&self) -> &Point<DatabaseUnitT> {
        &self.origin
    }

    const fn set_origin(&mut self, origin: Point<DatabaseUnitT>) {
        self.origin = origin;
    }

    pub const fn layer(&self) -> Layer {
        self.layer
    }

    pub const fn magnification(&self) -> f64 {
        self.magnification
    }

    pub const fn angle(&self) -> f64 {
        self.angle
    }

    pub const fn x_reflection(&self) -> bool {
        self.x_reflection
    }

    pub const fn vertical_presentation(&self) -> &presentation::VerticalPresentation {
        &self.vertical_presentation
    }

    pub const fn horizontal_presentation(&self) -> &presentation::HorizontalPresentation {
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
    fn transform_impl(&self, transformation: &Transformation) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_creation() {
        let text = Text::new(
            "Hello World".to_string(),
            Point::new(100, 200),
            5,
            2.0,
            45.0,
            true,
            presentation::VerticalPresentation::Top,
            presentation::HorizontalPresentation::Right,
        );

        assert_eq!(text.text(), "Hello World");
        assert_eq!(text.origin(), &Point::new(100, 200));
        assert_eq!(text.layer(), 5);
        assert_eq!(text.magnification(), 2.0);
        assert_eq!(text.angle(), 45.0);
        assert!(text.x_reflection());
        assert_eq!(
            text.vertical_presentation(),
            &presentation::VerticalPresentation::Top
        );
        assert_eq!(
            text.horizontal_presentation(),
            &presentation::HorizontalPresentation::Right
        );
    }

    #[test]
    fn test_text_default() {
        let text = Text::<DatabaseIntegerUnit>::default();

        assert_eq!(text.text(), "");
        assert_eq!(text.origin(), &Point::new(0, 0));
        assert_eq!(text.layer(), 0);
        assert_eq!(text.magnification(), 1.0);
        assert_eq!(text.angle(), 0.0);
        assert!(!text.x_reflection());
        assert_eq!(
            text.vertical_presentation(),
            &presentation::VerticalPresentation::default()
        );
        assert_eq!(
            text.horizontal_presentation(),
            &presentation::HorizontalPresentation::default()
        );
    }

    #[test]
    fn test_text_display() {
        let text = Text::new(
            "Test Text".to_string(),
            Point::new(10, 20),
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
    fn test_text_movable() {
        let text = Text::new(
            "Move Me".to_string(),
            Point::new(10, 20),
            0,
            1.0,
            0.0,
            false,
            presentation::VerticalPresentation::default(),
            presentation::HorizontalPresentation::default(),
        );

        let moved_text = text.move_to(Point::new(50, 60));
        assert_eq!(moved_text.origin(), &Point::new(50, 60));
        assert_eq!(moved_text.text(), "Move Me");
    }

    #[test]
    fn test_text_transformable_with_translation() {
        use crate::transformation::{Transformation, Translation};

        let text = Text::new(
            "Transform Me".to_string(),
            Point::new(0, 0),
            0,
            1.0,
            0.0,
            false,
            presentation::VerticalPresentation::default(),
            presentation::HorizontalPresentation::default(),
        );

        let translation = Translation::new(Point::new(10, 20));
        let transformation = Transformation::from(translation);

        let transformed_text = text.transform_impl(&transformation);
        assert_eq!(transformed_text.origin(), &Point::new(10, 20));
        assert_eq!(transformed_text.magnification(), 1.0);
    }

    #[test]
    fn test_text_transformable_with_scale() {
        use crate::transformation::{Scale, Transformation};

        let text = Text::new(
            "Scale Me".to_string(),
            Point::new(0, 0),
            0,
            2.0,
            0.0,
            false,
            presentation::VerticalPresentation::default(),
            presentation::HorizontalPresentation::default(),
        );

        let scale = Scale::new(1.5, Point::new(0, 0));
        let transformation = Transformation::from(scale);

        let transformed_text = text.transform_impl(&transformation);
        assert_eq!(transformed_text.magnification(), 3.0); // 2.0 * 1.5
    }

    #[test]
    fn test_text_transformable_with_rotation() {
        use crate::transformation::{Rotation, Transformation};

        let text = Text::new(
            "Rotate Me".to_string(),
            Point::new(0, 0),
            0,
            1.0,
            30.0,
            false,
            presentation::VerticalPresentation::default(),
            presentation::HorizontalPresentation::default(),
        );

        let rotation = Rotation::new(45.0, Point::new(0, 0));
        let transformation = Transformation::from(rotation);

        let transformed_text = text.transform_impl(&transformation);
        assert_eq!(transformed_text.angle(), 75.0); // 30.0 + 45.0
    }

    #[test]
    fn test_text_clone_and_partial_eq() {
        let text1 = Text::new(
            "Clone Test".to_string(),
            Point::new(5, 10),
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
            Point::new(5, 10),
            2,
            1.2,
            15.0,
            true,
            presentation::VerticalPresentation::Middle,
            presentation::HorizontalPresentation::Centre,
        );
        assert_ne!(text1, text3);
    }

    #[test]
    fn test_text_getters() {
        let text = Text::new(
            "Getter Test".to_string(),
            Point::new(100, 200),
            3,
            2.5,
            90.0,
            false,
            presentation::VerticalPresentation::Top,
            presentation::HorizontalPresentation::Left,
        );

        assert_eq!(text.text(), "Getter Test");
        assert_eq!(text.origin(), &Point::new(100, 200));
        assert_eq!(text.layer(), 3);
        assert_eq!(text.magnification(), 2.5);
        assert_eq!(text.angle(), 90.0);
        assert!(!text.x_reflection());
        assert_eq!(
            text.vertical_presentation(),
            &presentation::VerticalPresentation::Top
        );
        assert_eq!(
            text.horizontal_presentation(),
            &presentation::HorizontalPresentation::Left
        );
    }
}
