use std::sync::mpsc;

use chrono::{Datelike, Local, Timelike};
use rayon::prelude::*;

use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_structure_name, write_string_with_record_to_file, write_u16_array_to_file,
};
use crate::{
    Dimensions, Element, GdsBox, LayerMapping, Library, Movable, Node, Path, Point, Polygon,
    Reference, Text, Transformable, Transformation,
};

/// A named cell containing polygons, paths, boxes, nodes, texts, and references to other cells or elements.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Cell {
    name: String,
    elements: Vec<Element>,
}

impl Cell {
    /// Creates a new empty cell with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            elements: Vec::new(),
        }
    }

    /// Returns the cell name.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Returns an iterator over all elements in this cell.
    pub fn iter_elements(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter()
    }

    /// Returns a mutable iterator over all elements in this cell.
    pub fn iter_elements_mut(&mut self) -> impl Iterator<Item = &mut Element> {
        self.elements.iter_mut()
    }

    /// Returns an iterator over the polygons in this cell.
    pub fn polygons(&self) -> impl Iterator<Item = &Polygon> {
        self.elements.iter().filter_map(Element::as_polygon)
    }

    /// Returns an iterator over the paths in this cell.
    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        self.elements.iter().filter_map(Element::as_path)
    }

    /// Returns an iterator over the boxes in this cell.
    pub fn boxes(&self) -> impl Iterator<Item = &GdsBox> {
        self.elements.iter().filter_map(Element::as_box)
    }

    /// Returns an iterator over the nodes in this cell.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.elements.iter().filter_map(Element::as_node)
    }

    /// Returns an iterator over the texts in this cell.
    pub fn texts(&self) -> impl Iterator<Item = &Text> {
        self.elements.iter().filter_map(Element::as_text)
    }

    /// Returns an iterator over the references in this cell.
    pub fn references(&self) -> impl Iterator<Item = &Reference> {
        self.elements.iter().filter_map(Element::as_reference)
    }

    /// Returns the names of all cells referenced by this cell, recursively resolving through
    /// inline element wrappers.
    pub fn referenced_cell_names(&self) -> Vec<&str> {
        self.references()
            .filter_map(Reference::referenced_cell_name)
            .collect()
    }

    /// Adds an element (polygon, path, text, or reference) to the cell.
    pub fn add(&mut self, element: impl Into<Element>) {
        self.elements.push(element.into());
    }

    /// Converts all elements to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            name: self.name,
            elements: self
                .elements
                .into_iter()
                .map(Element::to_integer_unit)
                .collect(),
        }
    }

    /// Converts all elements to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            name: self.name,
            elements: self
                .elements
                .into_iter()
                .map(Element::to_float_unit)
                .collect(),
        }
    }

    /// Remaps layer/data type pairs on all elements in this cell using the given mapping.
    pub fn remap_layers(&mut self, mapping: &LayerMapping) {
        for element in &mut self.elements {
            element.remap_layers(mapping);
        }
    }

    /// Like [`get_elements`](Self::get_elements) but sends elements through a channel as they're
    /// produced, enabling progressive rendering. Stops early if the receiver is dropped.
    pub fn stream_elements(
        &self,
        depth: Option<usize>,
        library: &Library,
        tx: &mpsc::Sender<Element>,
    ) {
        let depth = depth.unwrap_or(usize::MAX);

        for element in &self.elements {
            if let Element::Reference(reference) = element {
                if reference
                    .clone()
                    .stream_flatten(Some(depth), library, tx)
                    .is_err()
                {
                    return;
                }
            } else if tx.send(element.clone()).is_err() {
                return;
            }
        }
    }

    /// Returns all elements in this cell, recursively flattening references up to the given depth.
    pub fn get_elements(&self, depth: Option<usize>, library: &Library) -> Vec<Element> {
        let depth = depth.unwrap_or(usize::MAX);
        let mut result: Vec<Element> = Vec::new();

        for element in &self.elements {
            if let Element::Reference(reference) = element {
                result.extend(reference.clone().flatten(Some(depth), library));
            } else {
                result.push(element.clone());
            }
        }

        result
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cell '{}' with {} polygon(s), {} path(s), {} box(es), {} node(s), {} text(s)",
            self.name,
            self.polygons().count(),
            self.paths().count(),
            self.boxes().count(),
            self.nodes().count(),
            self.texts().count(),
        )
    }
}

impl Transformable for Cell {
    fn transform_impl(self, transformation: &Transformation) -> Self {
        Self {
            name: self.name,
            elements: self
                .elements
                .into_iter()
                .map(|e| e.transform_impl(transformation))
                .collect(),
        }
    }
}

impl Dimensions for Cell {
    fn bounding_box(&self) -> (Point, Point) {
        let all_points: Vec<Point> = self
            .elements
            .iter()
            .filter(|e| !matches!(e, Element::Reference(_)))
            .flat_map(|e| {
                let (min, max) = e.bounding_box();
                vec![min, max]
            })
            .collect();

        crate::geometry::bounding_box(&all_points)
    }
}

impl Movable for Cell {
    fn move_to(self, target: crate::Point) -> Self {
        Self {
            name: self.name,
            elements: self
                .elements
                .into_iter()
                .map(|e| e.move_to(target))
                .collect(),
        }
    }
}

impl ToGds for Cell {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        validate_structure_name(&self.name)?;

        let now = Local::now();
        let timestamp = now.naive_utc();

        let cell_head = [
            28,
            combine_record_and_data_type(GDSRecord::BgnStr, GDSDataType::TwoByteSignedInteger),
            timestamp.year() as u16,
            timestamp.month() as u16,
            timestamp.day() as u16,
            timestamp.hour() as u16,
            timestamp.minute() as u16,
            timestamp.second() as u16,
            timestamp.year() as u16,
            timestamp.month() as u16,
            timestamp.day() as u16,
            timestamp.hour() as u16,
            timestamp.minute() as u16,
            timestamp.second() as u16,
        ];

        let mut buffer = Vec::new();

        write_u16_array_to_file(&mut buffer, &cell_head)?;

        write_string_with_record_to_file(&mut buffer, GDSRecord::StrName, &self.name)?;

        let element_bufs: Result<Vec<_>, _> = self
            .elements
            .par_iter()
            .map(|e| e.to_gds_impl(database_units))
            .collect();
        for b in element_bufs? {
            buffer.extend_from_slice(&b);
        }

        let cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        write_u16_array_to_file(&mut buffer, &cell_tail)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Layer};

    #[test]
    fn test_cell_new() {
        let cell = Cell::new("test_cell");
        assert_eq!(cell.name, "test_cell");
        assert_eq!(cell.polygons().next(), None);
        assert_eq!(cell.paths().next(), None);
        assert_eq!(cell.boxes().next(), None);
        assert_eq!(cell.nodes().next(), None);
        assert_eq!(cell.texts().next(), None);
        assert_eq!(cell.references().next(), None);
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.name, "");
        assert_eq!(cell.elements.len(), 0);
    }

    #[test]
    fn test_add_polygon() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::default();

        cell.add(polygon.clone());
        assert_eq!(cell.polygons().count(), 1);
        assert_eq!(cell.polygons().next().unwrap(), &polygon);
    }

    #[test]
    fn test_add_path() {
        let mut cell = Cell::new("test_cell");
        let path = Path::default();

        cell.add(path.clone());
        assert_eq!(cell.paths().count(), 1);
        assert_eq!(cell.paths().next().unwrap(), &path);
    }

    #[test]
    fn test_add_text() {
        let mut cell = Cell::new("test_cell");
        let text = Text::default();

        cell.add(text.clone());
        assert_eq!(cell.texts().count(), 1);
        assert_eq!(cell.texts().next().unwrap(), &text);
    }

    #[test]
    fn test_cell_display() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::default();
        cell.add(polygon);

        insta::assert_snapshot!(cell.to_string(), @"Cell 'test_cell' with 1 polygon(s), 0 path(s), 0 box(es), 0 node(s), 0 text(s)");
    }

    #[test]
    fn test_cell_to_integer_unit() {
        let mut cell = Cell::new("test");
        cell.add(Polygon::new(
            vec![
                Point::float(1.5, 2.5, 1e-6),
                Point::float(10.0, 0.0, 1e-6),
                Point::float(10.0, 10.0, 1e-6),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Path::new(
            vec![Point::float(0.0, 0.0, 1e-6)],
            Layer::new(0),
            DataType::new(0),
            None,
            None,
            None,
            None,
        ));
        cell.add(Text::default().set_origin(Point::float(5.0, 5.0, 1e-6)));

        let converted = cell.to_integer_unit();

        assert_eq!(converted.polygons().count(), 1);
        assert_eq!(converted.paths().count(), 1);
        assert_eq!(converted.texts().count(), 1);
        for point in converted.polygons().next().unwrap().points() {
            assert_eq!(*point, point.to_integer_unit());
        }
    }

    #[test]
    fn test_cell_to_float_unit() {
        let mut cell = Cell::new("test");
        cell.add(Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        let converted = cell.to_float_unit();

        assert_eq!(converted.polygons().count(), 1);
        for point in converted.polygons().next().unwrap().points() {
            assert_eq!(*point, point.to_float_unit());
        }
    }

    #[test]
    fn test_cell_get_elements() {
        let library = Library::new("main");

        let polygon = Polygon::default();

        let reference = Reference::new(polygon.clone());

        let mut cell = Cell::new("test_cell");

        let path = Path::default();

        let text = Text::default();

        cell.add(reference);
        cell.add(polygon);
        cell.add(path);
        cell.add(text);

        let elements = cell.get_elements(None, &library);

        insta::assert_debug_snapshot!(elements);

        let moved_cell = cell.move_to(Point::integer(5, 5, 1e-9));

        let moved_elements = moved_cell.get_elements(None, &library);

        insta::assert_debug_snapshot!(moved_elements);

        let rotated_cell = moved_cell.rotate(90.0, Point::integer(5, 5, 1e-9));

        let rotated_elements = rotated_cell.get_elements(None, &library);

        insta::assert_debug_snapshot!(rotated_elements);
    }

    #[test]
    fn test_cell_bounding_box() {
        let mut cell = Cell::new("test");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Path::new(
            vec![Point::integer(-5, 5, 1e-9), Point::integer(15, 20, 1e-9)],
            Layer::new(1),
            DataType::new(0),
            None,
            None,
            None,
            None,
        ));
        cell.add(Text::default().set_origin(Point::integer(3, -2, 1e-9)));

        let (min, max) = cell.bounding_box();
        assert_eq!(min, Point::integer(-5, -2, 1e-9));
        assert_eq!(max, Point::integer(15, 20, 1e-9));
    }

    #[test]
    fn test_cell_bounding_box_empty() {
        let cell = Cell::new("empty");
        let (min, max) = cell.bounding_box();
        assert_eq!(min, Point::default());
        assert_eq!(max, Point::default());
    }

    #[test]
    fn test_cell_bounding_box_polygons_only() {
        let mut cell = Cell::new("test");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(5, 0, 1e-9),
                Point::integer(5, 5, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Polygon::new(
            [
                Point::integer(10, 10, 1e-9),
                Point::integer(20, 10, 1e-9),
                Point::integer(20, 20, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        let (min, max) = cell.bounding_box();
        assert_eq!(min, Point::integer(0, 0, 1e-9));
        assert_eq!(max, Point::integer(20, 20, 1e-9));
    }

    #[test]
    fn stream_elements_matches_get_elements() {
        let mut library = Library::new("main");

        let polygon = Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let path = Path::new(
            vec![Point::integer(0, 0, 1e-9), Point::integer(5, 5, 1e-9)],
            Layer::new(2),
            DataType::new(0),
            None,
            None,
            None,
            None,
        );
        let text = Text::default().set_origin(Point::integer(3, 3, 1e-9));

        let mut inner = Cell::new("inner");
        inner.add(polygon.clone());
        library.add_cell(inner);

        let mut cell = Cell::new("test_cell");
        cell.add(polygon);
        cell.add(path);
        cell.add(text);
        cell.add(Reference::new("inner"));

        let expected = cell.get_elements(None, &library);

        let (tx, rx) = mpsc::channel();
        cell.stream_elements(None, &library, &tx);
        drop(tx);
        let streamed: Vec<Element> = rx.iter().collect();

        assert_eq!(expected.len(), streamed.len());
        assert_eq!(expected, streamed);
    }

    #[test]
    fn stream_elements_cancellation() {
        let library = Library::new("main");

        let mut cell = Cell::new("test_cell");
        cell.add(Polygon::new(
            vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        let (tx, rx) = mpsc::channel();
        drop(rx);
        cell.stream_elements(None, &library, &tx);
    }
}
