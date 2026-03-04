use std::sync::Arc;

use crate::elements::{Path, Polygon, Reference, Text};
use crate::traits::ToGds;
use crate::{Dimensions, Instance, Movable, Point, Transformable, Transformation};

/// A GDSII element: one of [`Path`], [`Polygon`], [`Text`], or [`Reference`].
#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    /// A path element.
    Path(Path),
    /// A polygon element.
    Polygon(Polygon),
    /// A text annotation element.
    Text(Text),
    /// A reference to another cell or element.
    Reference(Reference),
}

impl Element {
    /// Returns the inner [`Path`] if this is a `Path` variant, or `None`.
    pub fn as_path(&self) -> Option<&Path> {
        if let Self::Path(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the inner [`Polygon`] if this is a `Polygon` variant, or `None`.
    pub fn as_polygon(&self) -> Option<&Polygon> {
        if let Self::Polygon(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the inner [`Text`] if this is a `Text` variant, or `None`.
    pub fn as_text(&self) -> Option<&Text> {
        if let Self::Text(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the inner [`Reference`] if this is a `Reference` variant, or `None`.
    pub fn as_reference(&self) -> Option<&Reference> {
        if let Self::Reference(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Converts the inner element to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.to_integer_unit()),
            Self::Polygon(polygon) => Self::Polygon(polygon.to_integer_unit()),
            Self::Text(text) => Self::Text(text.to_integer_unit()),
            Self::Reference(reference) => Self::Reference(reference.to_integer_unit()),
        }
    }

    /// Converts the inner element to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.to_float_unit()),
            Self::Polygon(polygon) => Self::Polygon(polygon.to_float_unit()),
            Self::Text(text) => Self::Text(text.to_float_unit()),
            Self::Reference(reference) => Self::Reference(reference.to_float_unit()),
        }
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{path}"),
            Self::Polygon(polygon) => write!(f, "{polygon}"),
            Self::Text(text) => write!(f, "{text}"),
            Self::Reference(reference) => write!(f, "{reference}"),
        }
    }
}

macro_rules! impl_from_element_reference {
    ($($type:ident),*) => {
        $(
            impl From<$type> for Element {
                fn from(value: $type) -> Self {
                    Element::$type(value)
                }
            }

            impl From<$type> for Instance {
                fn from(value: $type) -> Self {
                    Element::from(value).into()
                }
            }
        )*
    };
}

impl_from_element_reference!(Path, Polygon, Text, Reference);

impl From<Element> for Instance {
    fn from(v: Element) -> Self {
        Self::Element(Arc::new(Box::new(v)))
    }
}

impl ToGds for Element {
    fn to_gds_impl(&self, scale: f64) -> Result<Vec<u8>, crate::error::GdsError> {
        match self {
            Self::Path(path) => path.to_gds_impl(scale),
            Self::Polygon(polygon) => polygon.to_gds_impl(scale),
            Self::Reference(reference) => reference.to_gds_impl(scale),
            Self::Text(text) => text.to_gds_impl(scale),
        }
    }
}

impl Transformable for Element {
    fn transform_impl(self, transformation: &Transformation) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.transform_impl(transformation)),
            Self::Polygon(polygon) => Self::Polygon(polygon.transform_impl(transformation)),
            Self::Reference(reference) => Self::Reference(reference.transform_impl(transformation)),
            Self::Text(text) => Self::Text(text.transform_impl(transformation)),
        }
    }
}

impl Movable for Element {
    fn move_to(self, target: Point) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.move_to(target)),
            Self::Polygon(polygon) => Self::Polygon(polygon.move_to(target)),
            Self::Reference(reference) => Self::Reference(reference.move_to(target)),
            Self::Text(text) => Self::Text(text.move_to(target)),
        }
    }
}

impl Dimensions for Element {
    fn bounding_box(&self) -> (Point, Point) {
        match self {
            Self::Path(path) => path.bounding_box(),
            Self::Polygon(polygon) => polygon.bounding_box(),
            Self::Text(text) => text.bounding_box(),
            Self::Reference(_) => (Point::default(), Point::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Grid, Point};

    const UNITS: f64 = 1e-9;

    fn p(x: i32, y: i32) -> Point {
        Point::integer(x, y, UNITS)
    }

    fn pf(x: f64, y: f64) -> Point {
        Point::float(x, y, 1e-6)
    }

    fn origin() -> Point {
        p(0, 0)
    }

    fn simple_polygon() -> Polygon {
        Polygon::new(vec![p(0, 0), p(10, 0), p(10, 10)], 1, 0)
    }

    fn simple_path() -> Path {
        Path::new(vec![p(0, 0), p(10, 10)], 1, 0, None, None)
    }

    fn simple_grid() -> Grid {
        Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(p(10, 0)))
            .with_spacing_y(Some(p(0, 10)))
    }

    fn simple_reference() -> Reference {
        Reference::new(simple_polygon())
    }

    fn simple_text() -> Text {
        Text::default()
    }

    fn all_elements() -> [Element; 4] {
        [
            simple_path().into(),
            simple_polygon().into(),
            simple_text().into(),
            simple_reference().with_grid(simple_grid()).into(),
        ]
    }

    #[test]
    fn test_element_from_path() {
        let path = simple_path();
        let element: Element = path.clone().into();

        assert!(element.as_polygon().is_none());
        assert!(element.as_reference().is_none());
        assert!(element.as_text().is_none());
        assert_eq!(element.as_path().unwrap(), &path);

        insta::assert_snapshot!(element.to_string(), @"Path with 2 points on layer 1 with data type 0, Square and width 0 (1.000e-9)");
    }

    #[test]
    fn test_element_from_polygon() {
        let polygon = simple_polygon();
        let element: Element = polygon.clone().into();

        assert!(element.as_path().is_none());
        assert_eq!(element.as_polygon().unwrap(), &polygon);

        insta::assert_snapshot!(element.to_string(), @"Polygon with 4 point(s), starting at (0 (1.000e-9), 0 (1.000e-9)) on layer 1, data type 0");
    }

    #[test]
    fn test_element_from_text() {
        let text = simple_text();
        let element: Element = text.clone().into();

        assert_eq!(element.as_text().unwrap(), &text);

        insta::assert_snapshot!(element.to_string(), @"Text '' vertical: Middle, horizontal: Centre at Point(0 (1.000e-9), 0 (1.000e-9))");
    }

    #[test]
    fn test_element_from_reference() {
        let reference = simple_reference().with_grid(simple_grid());
        let element: Element = reference.clone().into();

        assert_eq!(element.as_reference().unwrap(), &reference);

        insta::assert_snapshot!(element.to_string(), @"Reference to Element instance: Polygon with 4 point(s), starting at (0 (1.000e-9), 0 (1.000e-9)) on layer 1, data type 0 with grid Grid at Point(0 (1.000e-9), 0 (1.000e-9)) with 2 columns and 2 rows, spacing (Point(10 (1.000e-9), 0 (1.000e-9)), Point(0 (1.000e-9), 10 (1.000e-9))), magnification 1.0, angle 0.0, x_reflection false");
    }

    #[test]
    fn test_element_to_integer_unit_preserves_variant() {
        for element in all_elements() {
            let converted = element.clone().to_integer_unit();
            assert_eq!(
                std::mem::discriminant(&element),
                std::mem::discriminant(&converted)
            );
        }
    }

    #[test]
    fn test_element_to_float_unit_preserves_variant() {
        for element in all_elements() {
            let converted = element.clone().to_float_unit();
            assert_eq!(
                std::mem::discriminant(&element),
                std::mem::discriminant(&converted)
            );
        }
    }

    #[test]
    fn test_element_to_integer_unit_converts_points() {
        let polygon = Polygon::new([pf(1.5, 2.5), pf(10.0, 0.0), pf(10.0, 10.0)], 1, 0);
        let element: Element = polygon.into();
        let converted = element.to_integer_unit();

        for point in converted.as_polygon().unwrap().points() {
            assert_eq!(*point, point.to_integer_unit());
        }
    }

    #[test]
    fn test_element_to_float_unit_converts_points() {
        let element: Element = simple_polygon().into();
        let converted = element.to_float_unit();

        for point in converted.as_polygon().unwrap().points() {
            assert_eq!(*point, point.to_float_unit());
        }
    }

    #[test]
    fn test_element_transform_preserves_variant() {
        let centre = origin();
        for element in all_elements() {
            let rotated = element.clone().rotate(std::f64::consts::PI / 2.0, centre);
            assert_eq!(
                std::mem::discriminant(&element),
                std::mem::discriminant(&rotated)
            );
        }
    }

    #[test]
    fn test_element_move_to() {
        let element: Element = simple_polygon().into();
        let moved = element.move_to(p(20, 20));
        assert!(moved.as_polygon().is_some());
    }

    #[test]
    fn test_element_move_by() {
        let element: Element = simple_path().into();
        let moved = element.move_by(p(5, 5));
        assert!(moved.as_path().is_some());
    }

    #[test]
    fn test_element_bounding_box_polygon() {
        let element: Element = simple_polygon().into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, p(0, 0));
        assert_eq!(max, p(10, 10));
    }

    #[test]
    fn test_element_bounding_box_path() {
        let path = Path::new(vec![p(-5, 3), p(10, 7)], 1, 0, None, None);
        let element: Element = path.into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, p(-5, 3));
        assert_eq!(max, p(10, 7));
    }

    #[test]
    fn test_element_bounding_box_text() {
        let text = Text::default().set_origin(p(5, 10));
        let element: Element = text.into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, p(5, 10));
        assert_eq!(max, p(5, 10));
    }

    #[test]
    fn test_element_bounding_box_reference() {
        let element: Element = simple_reference().into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, Point::default());
        assert_eq!(max, Point::default());
    }
}
