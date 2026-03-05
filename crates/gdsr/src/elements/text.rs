use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_layer, validate_string_length, write_element_tail_to_file, write_points_to_file,
    write_string_with_record_to_file, write_transformation_to_file, write_u16_array_to_file,
};
use crate::{DataType, Dimensions, Layer, Movable, Point, Transformable};

// --- Presentation types ---

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalPresentation {
    Top = 0,
    #[default]
    Middle = 1,
    Bottom = 2,
}

impl std::fmt::Display for VerticalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Vertical {}", self.name())
    }
}

impl std::fmt::Debug for VerticalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl VerticalPresentation {
    pub fn new(value: i32) -> Result<Self, crate::error::GdsError> {
        match value {
            0 => Ok(Self::Top),
            1 => Ok(Self::Middle),
            2 => Ok(Self::Bottom),
            _ => Err(crate::error::GdsError::ValidationError {
                message: "Invalid value for VerticalPresentation".to_string(),
            }),
        }
    }

    pub const fn name(&self) -> &str {
        match self {
            Self::Top => "Top",
            Self::Middle => "Middle",
            Self::Bottom => "Bottom",
        }
    }

    pub const fn value(self) -> i32 {
        self as i32
    }

    pub fn values() -> Vec<Self> {
        vec![Self::Top, Self::Middle, Self::Bottom]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum HorizontalPresentation {
    Left = 0,
    #[default]
    Centre = 1,
    Right = 2,
}

impl std::fmt::Display for HorizontalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Horizontal {}", self.name())
    }
}

impl std::fmt::Debug for HorizontalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl HorizontalPresentation {
    pub fn new(value: i32) -> Result<Self, crate::error::GdsError> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Centre),
            2 => Ok(Self::Right),
            _ => Err(crate::error::GdsError::ValidationError {
                message: "Invalid value for HorizontalPresentation".to_string(),
            }),
        }
    }

    pub const fn name(&self) -> &str {
        match self {
            Self::Left => "Left",
            Self::Centre => "Centre",
            Self::Right => "Right",
        }
    }

    pub const fn value(self) -> i32 {
        self as i32
    }

    pub fn values() -> Vec<Self> {
        vec![Self::Left, Self::Centre, Self::Right]
    }
}

// --- Utility functions ---

pub const fn get_presentation_value(
    vertical_presentation: VerticalPresentation,
    horizontal_presentation: HorizontalPresentation,
) -> u16 {
    let vertical_value = vertical_presentation.value();
    let horizontal_value = horizontal_presentation.value();

    match (vertical_value, horizontal_value) {
        (0, 1) => 1,
        (0, 2) => 2,
        (1, 0) => 4,
        (1, 1) => 5,
        (1, 2) => 6,
        (2, 0) => 8,
        (2, 1) => 9,
        (2, 2) => 10,
        _ => 0,
    }
}

pub fn get_presentations_from_value(
    value: i16,
) -> Result<(VerticalPresentation, HorizontalPresentation), crate::error::GdsError> {
    let (vertical_value, horizontal_value) = match value {
        0 => (0, 0),
        1 => (0, 1),
        2 => (0, 2),
        4 => (1, 0),
        5 => (1, 1),
        6 => (1, 2),
        8 => (2, 0),
        9 => (2, 1),
        10 => (2, 2),
        _ => {
            return Err(crate::error::GdsError::ValidationError {
                message: "Invalid presentation value".to_string(),
            });
        }
    };

    let vertical_presentation = VerticalPresentation::new(vertical_value)?;
    let horizontal_presentation = HorizontalPresentation::new(horizontal_value)?;

    Ok((vertical_presentation, horizontal_presentation))
}

// --- Text struct ---

/// A text annotation placed at a specific point with configurable presentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    pub(crate) value: String,
    pub(crate) origin: Point,
    pub(crate) layer: Layer,
    pub(crate) datatype: DataType,
    pub(crate) magnification: f64,
    pub(crate) angle: f64,
    pub(crate) x_reflection: bool,
    pub(crate) vertical_presentation: VerticalPresentation,
    pub(crate) horizontal_presentation: HorizontalPresentation,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            value: String::new(),
            origin: Point::integer(0, 0, 1e-9),
            layer: Layer::new(0),
            datatype: DataType::new(0),
            magnification: 1.0,
            angle: 0.0,
            x_reflection: false,
            vertical_presentation: VerticalPresentation::default(),
            horizontal_presentation: HorizontalPresentation::default(),
        }
    }
}

impl Text {
    /// Creates a new text element with the given properties.
    pub fn new(
        text: &str,
        origin: Point,
        layer: Layer,
        datatype: DataType,
        magnification: f64,
        angle: f64,
        x_reflection: bool,
        vertical_presentation: VerticalPresentation,
        horizontal_presentation: HorizontalPresentation,
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

    /// Returns the text string.
    pub const fn text(&self) -> &String {
        &self.value
    }

    /// Sets the text string and returns the modified value.
    #[must_use]
    pub fn set_text(mut self, text: String) -> Self {
        self.value = text;
        self
    }

    /// Returns the origin point.
    pub const fn origin(&self) -> &Point {
        &self.origin
    }

    /// Sets the origin point and returns the modified value.
    #[must_use]
    pub fn set_origin(mut self, origin: Point) -> Self {
        self.origin = origin;
        self
    }

    /// Returns the layer number.
    pub const fn layer(&self) -> Layer {
        self.layer
    }

    /// Sets the layer number and returns the modified value.
    #[must_use]
    pub fn set_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Remaps the layer and data type using the given mapping.
    /// If the current (layer, datatype) pair is found in the mapping, it is replaced.
    pub fn remap_layers(&mut self, mapping: &crate::LayerMapping) {
        if let Some(&(new_layer, new_datatype)) = mapping.get(&(self.layer, self.datatype)) {
            self.layer = new_layer;
            self.datatype = new_datatype;
        }
    }

    /// Returns the magnification factor.
    pub const fn magnification(&self) -> f64 {
        self.magnification
    }

    /// Sets the magnification factor and returns the modified value.
    #[must_use]
    pub fn set_magnification(mut self, magnification: f64) -> Self {
        self.magnification = magnification;
        self
    }

    /// Returns the rotation angle in radians.
    pub const fn angle(&self) -> f64 {
        self.angle
    }

    /// Sets the rotation angle and returns the modified value.
    #[must_use]
    pub fn set_angle(mut self, angle: f64) -> Self {
        self.angle = angle;
        self
    }

    /// Returns whether x-axis reflection is enabled.
    pub const fn x_reflection(&self) -> bool {
        self.x_reflection
    }

    /// Sets x-axis reflection and returns the modified value.
    #[must_use]
    pub fn set_x_reflection(mut self, x_reflection: bool) -> Self {
        self.x_reflection = x_reflection;
        self
    }

    /// Returns the vertical text presentation alignment.
    pub const fn vertical_presentation(&self) -> &VerticalPresentation {
        &self.vertical_presentation
    }

    /// Sets the vertical presentation alignment and returns the modified value.
    #[must_use]
    pub fn set_vertical_presentation(
        mut self,
        vertical_presentation: VerticalPresentation,
    ) -> Self {
        self.vertical_presentation = vertical_presentation;
        self
    }

    /// Returns the horizontal text presentation alignment.
    pub const fn horizontal_presentation(&self) -> &HorizontalPresentation {
        &self.horizontal_presentation
    }

    /// Sets the horizontal presentation alignment and returns the modified value.
    #[must_use]
    pub fn set_horizontal_presentation(
        mut self,
        horizontal_presentation: HorizontalPresentation,
    ) -> Self {
        self.horizontal_presentation = horizontal_presentation;
        self
    }

    /// Converts origin to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            origin: self.origin.to_integer_unit(),
            ..self
        }
    }

    /// Converts origin to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            origin: self.origin.to_float_unit(),
            ..self
        }
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
        self.origin = transformation.apply_to_point(&self.origin);

        if let Some(scale) = &transformation.scale {
            self.magnification *= scale.factor();
        }

        if let Some(rotation) = &transformation.rotation {
            self.angle += rotation.angle();
        }

        if transformation.reflection.is_some() {
            self.x_reflection = !self.x_reflection;
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

impl Dimensions for Text {
    fn bounding_box(&self) -> (Point, Point) {
        (self.origin, self.origin)
    }
}

// --- ToGds impl ---

impl ToGds for Text {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        validate_layer(self.layer())?;
        validate_string_length(self.text())?;

        let mut buffer = Vec::new();

        let buffer_start = [
            4,
            combine_record_and_data_type(GDSRecord::Text, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer().value(),
            6,
            combine_record_and_data_type(GDSRecord::TextType, GDSDataType::TwoByteSignedInteger),
            0,
            6,
            combine_record_and_data_type(GDSRecord::Presentation, GDSDataType::BitArray),
            get_presentation_value(
                *self.vertical_presentation(),
                *self.horizontal_presentation(),
            ),
        ];

        write_u16_array_to_file(&mut buffer, &buffer_start)?;

        let angle = self.angle();
        let magnification = self.magnification();
        let x_reflection = self.x_reflection();

        write_transformation_to_file(&mut buffer, angle, magnification, x_reflection)?;

        write_points_to_file(&mut buffer, &[*self.origin()], database_units)?;

        write_string_with_record_to_file(&mut buffer, GDSRecord::String, self.text())?;

        write_element_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use insta::assert_debug_snapshot;

    use super::*;

    // Presentation tests

    #[test]
    fn test_vertical_presentation_new() {
        assert_eq!(
            VerticalPresentation::new(0).unwrap(),
            VerticalPresentation::Top
        );
        assert_eq!(
            VerticalPresentation::new(1).unwrap(),
            VerticalPresentation::Middle
        );
        assert_eq!(
            VerticalPresentation::new(2).unwrap(),
            VerticalPresentation::Bottom
        );
        assert!(VerticalPresentation::new(3).is_err());
        assert!(VerticalPresentation::new(-1).is_err());
    }

    #[test]
    fn test_vertical_presentation_name() {
        assert_eq!(VerticalPresentation::Top.name(), "Top");
        assert_eq!(VerticalPresentation::Middle.name(), "Middle");
        assert_eq!(VerticalPresentation::Bottom.name(), "Bottom");
    }

    #[test]
    fn test_vertical_presentation_value() {
        assert_eq!(VerticalPresentation::Top.value(), 0);
        assert_eq!(VerticalPresentation::Middle.value(), 1);
        assert_eq!(VerticalPresentation::Bottom.value(), 2);
    }

    #[test]
    fn test_vertical_presentation_values() {
        let values = VerticalPresentation::values();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&VerticalPresentation::Top));
        assert!(values.contains(&VerticalPresentation::Middle));
        assert!(values.contains(&VerticalPresentation::Bottom));
    }

    #[test]
    fn test_vertical_presentation_default() {
        assert_eq!(
            VerticalPresentation::default(),
            VerticalPresentation::Middle
        );
    }

    #[test]
    fn test_vertical_presentation_display_and_debug() {
        insta::assert_snapshot!(VerticalPresentation::Top.to_string(), @"Vertical Top");
        insta::assert_snapshot!(format!("{:?}", VerticalPresentation::Middle), @"Middle");
        insta::assert_snapshot!(VerticalPresentation::Bottom.to_string(), @"Vertical Bottom");
    }

    #[test]
    fn test_vertical_presentation_clone_copy_eq() {
        let vp = VerticalPresentation::Top;
        let cloned = vp;
        let copied = vp;

        assert_eq!(vp, cloned);
        assert_eq!(vp, copied);
        assert_ne!(vp, VerticalPresentation::Middle);
    }

    #[test]
    fn test_horizontal_presentation_new() {
        assert_eq!(
            HorizontalPresentation::new(0).unwrap(),
            HorizontalPresentation::Left
        );
        assert_eq!(
            HorizontalPresentation::new(1).unwrap(),
            HorizontalPresentation::Centre
        );
        assert_eq!(
            HorizontalPresentation::new(2).unwrap(),
            HorizontalPresentation::Right
        );
        assert!(HorizontalPresentation::new(3).is_err());
        assert!(HorizontalPresentation::new(-1).is_err());
    }

    #[test]
    fn test_horizontal_presentation_name() {
        assert_eq!(HorizontalPresentation::Left.name(), "Left");
        assert_eq!(HorizontalPresentation::Centre.name(), "Centre");
        assert_eq!(HorizontalPresentation::Right.name(), "Right");
    }

    #[test]
    fn test_horizontal_presentation_value() {
        assert_eq!(HorizontalPresentation::Left.value(), 0);
        assert_eq!(HorizontalPresentation::Centre.value(), 1);
        assert_eq!(HorizontalPresentation::Right.value(), 2);
    }

    #[test]
    fn test_horizontal_presentation_values() {
        let values = HorizontalPresentation::values();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&HorizontalPresentation::Left));
        assert!(values.contains(&HorizontalPresentation::Centre));
        assert!(values.contains(&HorizontalPresentation::Right));
    }

    #[test]
    fn test_horizontal_presentation_default() {
        assert_eq!(
            HorizontalPresentation::default(),
            HorizontalPresentation::Centre
        );
    }

    #[test]
    fn test_horizontal_presentation_display_and_debug() {
        insta::assert_snapshot!(HorizontalPresentation::Left.to_string(), @"Horizontal Left");
        insta::assert_snapshot!(format!("{:?}", HorizontalPresentation::Centre), @"Centre");
        insta::assert_snapshot!(HorizontalPresentation::Right.to_string(), @"Horizontal Right");
    }

    #[test]
    fn test_horizontal_presentation_clone_copy_eq() {
        let hp = HorizontalPresentation::Left;
        let cloned = hp;
        let copied = hp;

        assert_eq!(hp, cloned);
        assert_eq!(hp, copied);
        assert_ne!(hp, HorizontalPresentation::Centre);
    }

    #[test]
    fn test_presentation_error_messages() {
        let err = VerticalPresentation::new(10).unwrap_err();
        insta::assert_snapshot!(err.to_string(), @"Validation error: Invalid value for VerticalPresentation");
        let err = HorizontalPresentation::new(10).unwrap_err();
        insta::assert_snapshot!(err.to_string(), @"Validation error: Invalid value for HorizontalPresentation");
    }

    // Utils tests

    #[test]
    fn test_get_presentation_value_all_combinations() {
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(0).unwrap(),
                HorizontalPresentation::new(0).unwrap()
            ),
            0
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(0).unwrap(),
                HorizontalPresentation::new(1).unwrap()
            ),
            1
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(0).unwrap(),
                HorizontalPresentation::new(2).unwrap()
            ),
            2
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(1).unwrap(),
                HorizontalPresentation::new(0).unwrap()
            ),
            4
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(1).unwrap(),
                HorizontalPresentation::new(1).unwrap()
            ),
            5
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(1).unwrap(),
                HorizontalPresentation::new(2).unwrap()
            ),
            6
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(2).unwrap(),
                HorizontalPresentation::new(0).unwrap()
            ),
            8
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(2).unwrap(),
                HorizontalPresentation::new(1).unwrap()
            ),
            9
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(2).unwrap(),
                HorizontalPresentation::new(2).unwrap()
            ),
            10
        );
    }

    #[test]
    fn test_get_presentations_from_value_all_valid() {
        let test_cases = vec![
            (0, (0, 0)),
            (1, (0, 1)),
            (2, (0, 2)),
            (4, (1, 0)),
            (5, (1, 1)),
            (6, (1, 2)),
            (8, (2, 0)),
            (9, (2, 1)),
            (10, (2, 2)),
        ];

        for (input, (expected_v, expected_h)) in test_cases {
            let result = get_presentations_from_value(input);
            assert!(result.is_ok(), "Failed for input {input}");
            let (v, h) = result.unwrap();
            assert_eq!(v.value(), expected_v);
            assert_eq!(h.value(), expected_h);
        }
    }

    #[test]
    fn test_get_presentations_from_value_invalid() {
        assert!(get_presentations_from_value(3).is_err());
        assert!(get_presentations_from_value(7).is_err());
        assert!(get_presentations_from_value(11).is_err());
        assert!(get_presentations_from_value(-1).is_err());
        assert!(get_presentations_from_value(100).is_err());
    }

    #[test]
    fn test_roundtrip_presentation_conversion() {
        for v_val in 0..=2 {
            for h_val in 0..=2 {
                let v = VerticalPresentation::new(v_val).unwrap();
                let h = HorizontalPresentation::new(h_val).unwrap();
                let value = get_presentation_value(v, h);
                let (v_back, h_back) = get_presentations_from_value(value as i16).unwrap();
                assert_eq!(v, v_back);
                assert_eq!(h, h_back);
            }
        }
    }

    // Text struct tests

    #[test]
    fn test_text_creation() {
        let text = Text::new(
            "Hello World",
            Point::integer(100, 200, 1e-9),
            Layer::new(5),
            DataType::new(0),
            2.0,
            45.0,
            true,
            VerticalPresentation::Top,
            HorizontalPresentation::Right,
        );

        assert_eq!(text.text(), "Hello World");
        assert_eq!(text.origin(), &Point::integer(100, 200, 1e-9));
        assert_eq!(text.layer(), Layer::new(5));
        assert_eq!(text.magnification(), 2.0);
        assert_eq!(text.angle(), 45.0);
        assert!(text.x_reflection());
    }

    #[test]
    fn test_text_default() {
        let text = Text::default();

        assert_eq!(text.text(), "");
        assert_eq!(text.origin(), &Point::integer(0, 0, 1e-9));
        assert_eq!(text.layer(), Layer::new(0));
        assert_eq!(text.magnification(), 1.0);
        assert_eq!(text.angle(), 0.0);
        assert!(!text.x_reflection());
    }

    #[test]
    fn test_text_display() {
        let text = Text::new(
            "Test Text",
            Point::integer(10, 20, 1e-9),
            Layer::new(1),
            DataType::new(0),
            1.5,
            30.0,
            false,
            VerticalPresentation::Bottom,
            HorizontalPresentation::Left,
        );

        insta::assert_snapshot!(text.to_string(), @"Text 'Test Text' vertical: Bottom, horizontal: Left at Point(10 (1.000e-9), 20 (1.000e-9))");
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
        let text = Text::default().set_layer(Layer::new(10));
        assert_eq!(text.layer(), Layer::new(10));
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
        let text = Text::default().set_vertical_presentation(VerticalPresentation::Top);
        assert_eq!(text.vertical_presentation(), &VerticalPresentation::Top);
    }

    #[test]
    fn test_set_horizontal_presentation() {
        let text = Text::default().set_horizontal_presentation(HorizontalPresentation::Right);
        assert_eq!(
            text.horizontal_presentation(),
            &HorizontalPresentation::Right
        );
    }

    #[test]
    fn test_setter_chaining() {
        let text = Text::default()
            .set_text("Chained".to_string())
            .set_origin(Point::integer(100, 200, 1e-9))
            .set_layer(Layer::new(5))
            .set_magnification(2.0)
            .set_angle(45.0)
            .set_x_reflection(true)
            .set_vertical_presentation(VerticalPresentation::Bottom)
            .set_horizontal_presentation(HorizontalPresentation::Left);

        assert_eq!(text.text(), "Chained");
        assert_eq!(text.origin(), &Point::integer(100, 200, 1e-9));
        assert_eq!(text.layer(), Layer::new(5));
        assert_eq!(text.magnification(), 2.0);
        assert_eq!(text.angle(), 45.0);
        assert!(text.x_reflection());
        assert_eq!(text.vertical_presentation(), &VerticalPresentation::Bottom);
        assert_eq!(
            text.horizontal_presentation(),
            &HorizontalPresentation::Left
        );
    }

    #[test]
    fn test_text_to_integer_unit() {
        let text = Text::default().set_origin(Point::float(1.5, 2.5, 1e-6));
        let converted = text.to_integer_unit();

        assert_eq!(
            *converted.origin(),
            Point::float(1.5, 2.5, 1e-6).to_integer_unit()
        );
    }

    #[test]
    fn test_text_to_float_unit() {
        let text = Text::default().set_origin(Point::integer(100, 200, 1e-9));
        let converted = text.to_float_unit();

        assert_eq!(
            *converted.origin(),
            Point::integer(100, 200, 1e-9).to_float_unit()
        );
    }

    #[test]
    fn test_text_rotate() {
        let text = Text::default();

        let rotated_text = text.rotate(PI / 2.0, Point::origin());

        assert_debug_snapshot!(rotated_text, @r#"
        Text {
            value: "",
            origin: Point {
                x: Integer(
                    IntegerUnit {
                        value: 0,
                        units: 1e-9,
                    },
                ),
                y: Integer(
                    IntegerUnit {
                        value: 0,
                        units: 1e-9,
                    },
                ),
            },
            layer: Layer(
                0,
            ),
            datatype: DataType(
                0,
            ),
            magnification: 1.0,
            angle: 1.5707963267948966,
            x_reflection: false,
            vertical_presentation: Middle,
            horizontal_presentation: Centre,
        }
        "#);
    }

    #[test]
    fn test_text_reflect() {
        let text = Text::default()
            .set_origin(Point::integer(10, 20, 1e-9))
            .set_x_reflection(false);

        let reflected = text.reflect(0.0, Point::origin());

        assert!(reflected.x_reflection());
        assert_eq!(reflected.origin(), &Point::integer(10, -20, 1e-9));

        let reflected_again = reflected.reflect(0.0, Point::origin());

        assert!(!reflected_again.x_reflection());
        assert_eq!(reflected_again.origin(), &Point::integer(10, 20, 1e-9));
    }

    #[test]
    fn test_text_scale() {
        let text = Text::default()
            .set_origin(Point::integer(10, 20, 1e-9))
            .set_magnification(1.5);

        let centre = Point::integer(0, 0, 1e-9);
        let scaled = text.scale(2.0, centre);

        assert_eq!(scaled.magnification(), 3.0);
        assert_eq!(scaled.origin(), &Point::integer(20, 40, 1e-9));
    }

    #[test]
    fn test_text_bounding_box() {
        let origin = Point::integer(10, 20, 1e-9);
        let text = Text::default().set_origin(origin);
        let (min, max) = text.bounding_box();
        assert_eq!(min, origin);
        assert_eq!(max, origin);
    }

    #[test]
    fn test_text_bounding_box_default() {
        let text = Text::default();
        let (min, max) = text.bounding_box();
        assert_eq!(min, Point::integer(0, 0, 1e-9));
        assert_eq!(max, Point::integer(0, 0, 1e-9));
    }

    #[test]
    fn test_text_reflect_scale_rotate() {
        let text = Text::default()
            .set_origin(Point::integer(10, 0, 1e-9))
            .set_magnification(1.0)
            .set_x_reflection(false);

        let transformed = text
            .reflect(0.0, Point::origin())
            .scale(2.0, Point::origin())
            .rotate(PI / 2.0, Point::origin());

        assert!(transformed.x_reflection());
        assert_eq!(transformed.magnification(), 2.0);
        assert_eq!(transformed.angle(), PI / 2.0);

        assert_debug_snapshot!(transformed, @r#"
        Text {
            value: "",
            origin: Point {
                x: Integer(
                    IntegerUnit {
                        value: 0,
                        units: 1e-9,
                    },
                ),
                y: Integer(
                    IntegerUnit {
                        value: 20,
                        units: 1e-9,
                    },
                ),
            },
            layer: Layer(
                0,
            ),
            datatype: DataType(
                0,
            ),
            magnification: 2.0,
            angle: 1.5707963267948966,
            x_reflection: true,
            vertical_presentation: Middle,
            horizontal_presentation: Centre,
        }
        "#);
    }
}
