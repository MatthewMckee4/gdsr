use std::sync::Arc;

use crate::Cell;
use crate::elements::Element;
// Note: Reference is defined in parent module, so we can't import it here to avoid circular dependency

#[derive(Clone, Debug, PartialEq)]
pub enum Instance {
    Cell(String),
    Element(Arc<Box<Element>>),
}

impl Instance {
    pub fn as_cell(&self) -> Option<&String> {
        if let Self::Cell(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_element(&self) -> Option<&Arc<Box<Element>>> {
        if let Self::Element(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::Cell(String::new())
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cell(name) => write!(f, "Cell instance: {name}"),
            Self::Element(element) => write!(f, "Element instance: {element}"),
        }
    }
}

// From implementations for common element types
impl From<&Cell> for Instance {
    fn from(value: &Cell) -> Self {
        Self::Cell(value.name().to_string())
    }
}

impl From<String> for Instance {
    fn from(value: String) -> Self {
        Self::Cell(value)
    }
}

impl From<&str> for Instance {
    fn from(value: &str) -> Self {
        Self::Cell(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Path, Point, Polygon, Text};

    #[test]
    fn test_instance_cell() {
        let instance = Instance::from("test_cell");
        assert_eq!(instance, Instance::Cell("test_cell".to_string()));
    }

    #[test]
    fn test_instance_from_string() {
        let instance = Instance::from(String::from("my_cell"));
        assert_eq!(instance, Instance::Cell("my_cell".to_string()));
    }

    #[test]
    fn test_instance_from_polygon() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        assert!(Instance::from(polygon).as_element().is_some());
    }

    #[test]
    fn test_instance_default() {
        let instance = Instance::default();
        assert_eq!(instance, Instance::Cell(String::new()));
    }

    #[test]
    fn test_instance_display() {
        let instance = Instance::from("test_cell");
        let display_str = format!("{instance}");
        assert!(display_str.contains("Cell instance"));
        assert!(display_str.contains("test_cell"));
    }

    #[test]
    fn test_instance_clone() {
        let instance = Instance::from("test_cell");
        let cloned = instance.clone();
        assert_eq!(instance, cloned);
    }

    #[test]
    fn test_instance_from_path() {
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)],
            1,
            0,
            None,
            None,
        );

        let instance = Instance::from(path);

        assert!(instance.as_element().unwrap().as_path().is_some());
    }

    #[test]
    fn test_instance_from_text() {
        use crate::{HorizontalPresentation, VerticalPresentation};

        let text = Text::new(
            "test",
            Point::integer(0, 0, 1e-9),
            1,
            0,
            1.0,
            0.0,
            false,
            VerticalPresentation::Middle,
            HorizontalPresentation::Centre,
        );

        let instance = Instance::from(text);

        assert!(instance.as_element().unwrap().as_text().is_some());
    }

    #[test]
    fn test_instance_from_cell() {
        let cell = Cell::new("my_cell");
        let instance = Instance::from(&cell);

        assert_eq!(instance, Instance::Cell("my_cell".to_string()));
    }

    #[test]
    fn test_instance_display_element() {
        let polygon = Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            1,
            0,
        );
        let instance = Instance::from(polygon);
        let display_str = format!("{instance}");
        assert!(display_str.contains("Element instance"));
        assert!(display_str.contains("Polygon"));
    }
}
