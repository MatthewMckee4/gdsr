use std::fmt;

use crate::{Element, GdsBox, Node, Path, Polygon, Reference, Text};

/// A GDS II element property represented by a `PROPATTR`/`PROPVALUE` record pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    attribute: u16,
    value: String,
}

#[derive(Clone, Default)]
pub struct PropertyStore(Option<Box<PropertyList>>);

#[derive(Clone)]
struct PropertyList(Vec<Property>);

impl PropertyStore {
    fn as_slice(&self) -> &[Property] {
        self.0.as_deref().map_or(&[], |list| list.0.as_slice())
    }

    fn as_mut_vec(&mut self) -> &mut Vec<Property> {
        &mut self
            .0
            .get_or_insert_with(|| Box::new(PropertyList(Vec::new())))
            .as_mut()
            .0
    }
}

impl fmt::Debug for PropertyStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(formatter)
    }
}

impl PartialEq for PropertyStore {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for PropertyStore {}

impl Property {
    /// Creates a property with the given numeric attribute and string value.
    pub fn new(attribute: u16, value: impl Into<String>) -> Self {
        Self {
            attribute,
            value: value.into(),
        }
    }

    /// Returns the numeric property attribute.
    pub const fn attribute(&self) -> u16 {
        self.attribute
    }

    /// Returns the property value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

macro_rules! impl_properties {
    ($($element:ty),+ $(,)?) => {
        $(
            impl $element {
                /// Returns this element's ordered properties.
                pub fn properties(&self) -> &[Property] {
                    self.properties.as_slice()
                }

                /// Returns this element's ordered properties mutably.
                pub fn properties_mut(&mut self) -> &mut Vec<Property> {
                    self.properties.as_mut_vec()
                }
            }
        )+
    };
}

impl_properties!(Polygon, Path, Text, Reference, GdsBox, Node);

impl Element {
    /// Returns the inner element's ordered properties.
    pub fn properties(&self) -> &[Property] {
        match self {
            Self::Path(path) => path.properties(),
            Self::Polygon(polygon) => polygon.properties(),
            Self::Box(gds_box) => gds_box.properties(),
            Self::Node(node) => node.properties(),
            Self::Text(text) => text.properties(),
            Self::Reference(reference) => reference.properties(),
        }
    }

    /// Returns the inner element's ordered properties mutably.
    pub fn properties_mut(&mut self) -> &mut Vec<Property> {
        match self {
            Self::Path(path) => path.properties_mut(),
            Self::Polygon(polygon) => polygon.properties_mut(),
            Self::Box(gds_box) => gds_box.properties_mut(),
            Self::Node(node) => node.properties_mut(),
            Self::Text(text) => text.properties_mut(),
            Self::Reference(reference) => reference.properties_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;
    use crate::{DataType, Layer, Point};

    #[test]
    fn property_accessors_preserve_order_and_duplicate_attributes() {
        let mut element = Element::Polygon(Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(0, 10, 1e-9),
            ],
            Layer::default(),
            DataType::default(),
        ));

        element
            .properties_mut()
            .extend([Property::new(7, "net-a"), Property::new(7, "net-b")]);

        assert_eq!(element.properties()[0].attribute(), 7);
        assert_eq!(element.properties()[0].value(), "net-a");
        assert_eq!(element.properties()[1].value(), "net-b");
    }

    #[test]
    fn empty_property_storage_is_pointer_sized() {
        let mut allocated_empty = PropertyStore::default();
        allocated_empty.as_mut_vec();

        assert_eq!(size_of::<PropertyStore>(), size_of::<usize>());
        assert_eq!(allocated_empty, PropertyStore::default());
    }
}
