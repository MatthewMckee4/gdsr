use crate::{
    Movable, Point, Transformable, Transformation,
    elements::{Path, Polygon, Reference, Text},
    traits::ToGds,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Path(Path),
    Polygon(Polygon),
    Text(Text),
    Reference(Reference),
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
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        match self {
            Self::Path(path) => Self::Path(path.transform_impl(transformation)),
            Self::Polygon(polygon) => Self::Polygon(polygon.transform_impl(transformation)),
            Self::Reference(reference) => Self::Reference(reference.transform_impl(transformation)),
            Self::Text(text) => Self::Text(text.transform_impl(transformation)),
        }
    }
}

impl Movable for Element {
    fn move_to(&self, target: Point) -> Self {
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
    use crate::Point;

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

        match element {
            Element::Path(p) => assert_eq!(p, path),
            _ => panic!("Expected Path element"),
        }
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

        match element {
            Element::Polygon(p) => assert_eq!(p, polygon),
            _ => panic!("Expected Polygon element"),
        }
    }

    #[test]
    fn test_element_display() {
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

        let display_str = format!("{element}");
        assert!(display_str.contains("Polygon"));
    }

    #[test]
    fn test_element_clone() {
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
            1,
            0,
            None,
            None,
        );
        let element: Element = path.into();
        let cloned = element.clone();

        assert_eq!(element, cloned);
    }
}
