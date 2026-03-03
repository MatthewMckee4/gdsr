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
    fn to_gds_impl(
        &self,
        buffer: &mut impl std::io::Write,
        scale: f64,
    ) -> Result<(), crate::error::GdsError> {
        match self {
            Self::Path(path) => path.to_gds_impl(buffer, scale),
            Self::Polygon(polygon) => polygon.to_gds_impl(buffer, scale),
            Self::Reference(reference) => reference.to_gds_impl(buffer, scale),
            Self::Text(text) => text.to_gds_impl(buffer, scale),
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

    #[test]
    fn test_element_from_path() {
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
            1,
            0,
            None,
            None,
        );
        let element: Element = path.clone().into();

        assert!(element.as_polygon().is_none());
        assert!(element.as_reference().is_none());
        assert!(element.as_text().is_none());

        let p = element.as_path().unwrap().clone();

        assert_eq!(p, path);

        insta::assert_snapshot!(element.to_string(), @"Path with 2 points on layer 1 with data type 0, Square and width 0 (1.000e-9)");
    }

    #[test]
    fn test_element_from_polygon() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let element: Element = polygon.clone().into();

        assert!(element.as_path().is_none());

        let p = element.as_polygon().unwrap().clone();

        assert_eq!(p, polygon);

        insta::assert_snapshot!(element.to_string(), @"Polygon with 4 point(s), starting at (0 (1.000e-9), 0 (1.000e-9)) on layer 1, data type 0");
    }

    #[test]
    fn test_element_from_text() {
        let text = Text::default();

        let element: Element = text.clone().into();

        let t = element.as_text().unwrap().clone();

        assert_eq!(t, text);

        insta::assert_snapshot!(element.to_string(), @"Text '' vertical: Middle, horizontal: Centre at Point(0 (1.000e-9), 0 (1.000e-9))");
    }

    #[test]
    fn test_element_from_reference() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );

        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon.clone()).with_grid(grid);

        let element: Element = reference.clone().into();

        let r = element.as_reference().unwrap().clone();

        assert_eq!(r, reference);

        insta::assert_snapshot!(element.to_string(), @"Reference to Element instance: Polygon with 4 point(s), starting at (0 (1.000e-9), 0 (1.000e-9)) on layer 1, data type 0 with grid Grid at Point(0 (1.000e-9), 0 (1.000e-9)) with 2 columns and 2 rows, spacing (Point(10 (1.000e-9), 0 (1.000e-9)), Point(0 (1.000e-9), 10 (1.000e-9))), magnification 1.0, angle 0.0, x_reflection false");
    }

    #[test]
    fn test_element_to_integer_unit_polygon() {
        let polygon = Polygon::new(
            [
                Point::float(1.5, 2.5, 1e-6),
                Point::float(10.0, 0.0, 1e-6),
                Point::float(10.0, 10.0, 1e-6),
            ],
            1,
            0,
        );
        let element: Element = polygon.into();
        let converted = element.to_integer_unit();

        assert!(converted.as_polygon().is_some());
        for point in converted.as_polygon().unwrap().points() {
            assert_eq!(*point, point.to_integer_unit());
        }
    }

    #[test]
    fn test_element_to_float_unit_path() {
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
            1,
            0,
            None,
            Some(crate::Unit::default_integer(10)),
        );
        let element: Element = path.into();
        let converted = element.to_float_unit();

        assert!(converted.as_path().is_some());
        for point in converted.as_path().unwrap().points() {
            assert_eq!(*point, point.to_float_unit());
        }
    }

    #[test]
    fn test_element_to_integer_unit_text() {
        let text = Text::default();
        let element: Element = text.into();
        let converted = element.to_integer_unit();

        assert!(converted.as_text().is_some());
    }

    #[test]
    fn test_element_to_float_unit_reference() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let reference = Reference::new(polygon);
        let element: Element = reference.into();
        let converted = element.to_float_unit();

        assert!(converted.as_reference().is_some());
    }

    #[test]
    fn test_element_transform_polygon() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let element: Element = polygon.into();

        let centre = Point::integer(5, 5, 1e-9);
        let transformed = element.clone().rotate(std::f64::consts::PI / 2.0, centre);

        assert!(transformed.as_polygon().is_some());
    }

    #[test]
    fn test_element_transform_path() {
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
            1,
            0,
            None,
            None,
        );
        let element: Element = path.into();

        let centre = Point::integer(0, 0, 1e-9);
        let transformed = element.clone().scale(2.0, centre);

        assert!(transformed.as_path().is_some());
    }

    #[test]
    fn test_element_transform_text() {
        let text = Text::default();
        let element: Element = text.into();

        let centre = Point::integer(0, 0, 1e-9);
        let transformed = element.clone().reflect(0.0, centre);

        assert!(transformed.as_text().is_some());
    }

    #[test]
    fn test_element_transform_reference() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );

        let grid = Grid::default()
            .with_columns(2)
            .with_rows(2)
            .with_spacing_x(Some(Point::integer(10, 0, 1e-9)))
            .with_spacing_y(Some(Point::integer(0, 10, 1e-9)));

        let reference = Reference::new(polygon).with_grid(grid);
        let element: Element = reference.into();

        let delta = Point::integer(5, 5, 1e-9);
        let transformed = element.clone().translate(delta);

        assert!(transformed.as_reference().is_some());
    }

    #[test]
    fn test_element_move_to() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let element: Element = polygon.into();

        let target = Point::integer(20, 20, 1e-9);
        let moved = element.move_to(target);

        assert!(moved.as_polygon().is_some());
    }

    #[test]
    fn test_element_move_by() {
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
            1,
            0,
            None,
            None,
        );
        let element: Element = path.into();

        let delta = Point::integer(5, 5, 1e-9);
        let moved = element.move_by(delta);

        assert!(moved.as_path().is_some());
    }

    #[test]
    fn test_element_bounding_box_polygon() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let element: Element = polygon.into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, Point::integer(0, 0, 1e-9));
        assert_eq!(max, Point::integer(10, 10, 1e-9));
    }

    #[test]
    fn test_element_bounding_box_path() {
        let path = Path::new(
            vec![Point::integer(-5, 3, 1e-9), Point::integer(10, 7, 1e-9)],
            1,
            0,
            None,
            None,
        );
        let element: Element = path.into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, Point::integer(-5, 3, 1e-9));
        assert_eq!(max, Point::integer(10, 7, 1e-9));
    }

    #[test]
    fn test_element_bounding_box_text() {
        let text = Text::default().set_origin(Point::integer(5, 10, 1e-9));
        let element: Element = text.into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, Point::integer(5, 10, 1e-9));
        assert_eq!(max, Point::integer(5, 10, 1e-9));
    }

    #[test]
    fn test_element_to_integer_unit_path() {
        let path = Path::new(
            vec![Point::float(1.5, 2.5, 1e-6), Point::float(10.0, 0.0, 1e-6)],
            1,
            0,
            None,
            Some(crate::Unit::default_integer(10)),
        );
        let element: Element = path.into();
        let converted = element.to_integer_unit();

        assert!(converted.as_path().is_some());
        for point in converted.as_path().unwrap().points() {
            assert_eq!(*point, point.to_integer_unit());
        }
    }

    #[test]
    fn test_element_to_integer_unit_reference() {
        let polygon = Polygon::new(
            [
                Point::float(1.5, 2.5, 1e-6),
                Point::float(10.0, 0.0, 1e-6),
                Point::float(10.0, 10.0, 1e-6),
            ],
            1,
            0,
        );
        let reference = Reference::new(polygon);
        let element: Element = reference.into();
        let converted = element.to_integer_unit();

        assert!(converted.as_reference().is_some());
    }

    #[test]
    fn test_element_to_float_unit_polygon() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let element: Element = polygon.into();
        let converted = element.to_float_unit();

        assert!(converted.as_polygon().is_some());
        for point in converted.as_polygon().unwrap().points() {
            assert_eq!(*point, point.to_float_unit());
        }
    }

    #[test]
    fn test_element_to_float_unit_text() {
        let text = Text::default();
        let element: Element = text.into();
        let converted = element.to_float_unit();

        assert!(converted.as_text().is_some());
    }

    #[test]
    fn test_element_bounding_box_reference() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let reference = Reference::new(polygon);
        let element: Element = reference.into();
        let (min, max) = element.bounding_box();
        assert_eq!(min, Point::default());
        assert_eq!(max, Point::default());
    }
}
