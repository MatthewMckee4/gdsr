use std::sync::Arc;

use crate::Cell;
use crate::elements::{Element, Path, Polygon, Text};
// Note: Reference is defined in parent module, so we can't import it here to avoid circular dependency

#[derive(Clone, Debug, PartialEq)]
pub enum Instance {
    Cell(String),
    Element(Arc<Box<Element>>),
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
impl From<Polygon> for Instance {
    fn from(value: Polygon) -> Self {
        Self::Element(Arc::new(Box::new(Element::Polygon(value))))
    }
}

impl From<Path> for Instance {
    fn from(value: Path) -> Self {
        Self::Element(Arc::new(Box::new(Element::Path(value))))
    }
}

impl From<Text> for Instance {
    fn from(value: Text) -> Self {
        Self::Element(Arc::new(Box::new(Element::Text(value))))
    }
}

// TODO: Re-add after Reference is updated
// impl From<Reference> for Instance {
//     fn from(value: Reference) -> Self {
//         Instance::Element(Arc::new(Box::new(Element::Reference(value))))
//     }
// }

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
    use crate::Point;

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
        let instance = Instance::from(polygon);

        match instance {
            Instance::Element(_) => {}
            Instance::Cell(_) => panic!("Expected Element variant"),
        }
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
}
