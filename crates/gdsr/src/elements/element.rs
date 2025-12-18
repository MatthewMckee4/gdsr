use crate::elements::{Path, Polygon, Reference, Text};
use crate::traits::ToGds;
use crate::{Movable, Point, Transformable, Transformation};

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Path(Path),
    Polygon(Polygon),
    Text(Text),
    Reference(Reference),
}

impl Element {
    pub fn as_path(&self) -> Option<&Path> {
        if let Self::Path(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_polygon(&self) -> Option<&Polygon> {
        if let Self::Polygon(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<&Text> {
        if let Self::Text(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_reference(&self) -> Option<&Reference> {
        if let Self::Reference(v) = self {
            Some(v)
        } else {
            None
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

// From implementations for ergonomic construction
impl From<Path> for Element {
    fn from(path: Path) -> Self {
        Self::Path(path)
    }
}

impl From<Polygon> for Element {
    fn from(polygon: Polygon) -> Self {
        Self::Polygon(polygon)
    }
}

impl From<Text> for Element {
    fn from(text: Text) -> Self {
        Self::Text(text)
    }
}

impl From<Reference> for Element {
    fn from(reference: Reference) -> Self {
        Self::Reference(reference)
    }
}

impl ToGds for Element {
    fn to_gds_impl(&self, file: &mut std::fs::File, scale: f64) -> std::io::Result<()> {
        match self {
            Self::Path(path) => path.to_gds_impl(file, scale),
            Self::Polygon(polygon) => polygon.to_gds_impl(file, scale),
            Self::Reference(reference) => reference.to_gds_impl(file, scale),
            Self::Text(text) => text.to_gds_impl(file, scale),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Grid, HorizontalPresentation, Point, VerticalPresentation};

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

        insta::assert_snapshot!(element.to_string(), @"Path with 2 points on layer 1 with data type 0, Square and width 0");
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
        let text = Text::new(
            "text",
            Point::integer(0, 0, 1e-9),
            1,
            1.0,
            0.0,
            false,
            VerticalPresentation::Middle,
            HorizontalPresentation::Centre,
        );
        let element: Element = text.clone().into();

        let t = element.as_text().unwrap().clone();

        assert_eq!(t, text);

        insta::assert_snapshot!(element.to_string(), @"Text 'text' vertical: Middle, horizontal: Centre at Point { x: Integer { value: 0, units: 1e-9 }, y: Integer { value: 0, units: 1e-9 } }");
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

        let grid = Grid::new(
            Point::integer(0, 0, 1e-9),
            2,
            2,
            Point::integer(10, 0, 1e-9),
            Point::integer(0, 10, 1e-9),
            1.0,
            0.0,
            false,
        );

        let reference = Reference::new(polygon.clone(), grid);

        let element: Element = reference.clone().into();

        let r = element.as_reference().unwrap().clone();

        assert_eq!(r, reference);

        insta::assert_snapshot!(element.to_string(), @"Reference to Element instance: Polygon with 4 point(s), starting at (0 (1.000e-9), 0 (1.000e-9)) on layer 1, data type 0 with grid Grid at Point(0 (1.000e-9), 0 (1.000e-9)) with 2 columns and 2 rows, spacing (Point(10 (1.000e-9), 0 (1.000e-9)), Point(0 (1.000e-9), 10 (1.000e-9))), magnification 1.0, angle 0.0, x_reflection false");
    }
}
