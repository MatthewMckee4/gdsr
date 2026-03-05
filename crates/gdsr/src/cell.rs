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
    Dimensions, Element, GdsBox, Library, Movable, Path, Point, Polygon, Reference, Text,
    Transformable, Transformation,
};

/// A named cell containing polygons, paths, boxes, texts, and references to other cells or elements.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Cell {
    name: String,
    polygons: Vec<Polygon>,
    paths: Vec<Path>,
    boxes: Vec<GdsBox>,
    texts: Vec<Text>,
    references: Vec<Reference>,
}

impl Cell {
    /// Creates a new empty cell with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            polygons: Vec::new(),
            paths: Vec::new(),
            boxes: Vec::new(),
            texts: Vec::new(),
            references: Vec::new(),
        }
    }

    /// Returns the cell name.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Returns the polygons in this cell.
    pub const fn polygons(&self) -> &Vec<Polygon> {
        &self.polygons
    }

    /// Returns the paths in this cell.
    pub const fn paths(&self) -> &Vec<Path> {
        &self.paths
    }

    /// Returns the boxes in this cell.
    pub const fn boxes(&self) -> &Vec<GdsBox> {
        &self.boxes
    }

    /// Returns the texts in this cell.
    pub const fn texts(&self) -> &Vec<Text> {
        &self.texts
    }

    /// Returns the references in this cell.
    pub const fn references(&self) -> &Vec<Reference> {
        &self.references
    }

    /// Returns the names of all cells referenced by this cell, recursively resolving through
    /// inline element wrappers.
    pub fn referenced_cell_names(&self) -> Vec<&str> {
        self.references
            .iter()
            .filter_map(Reference::referenced_cell_name)
            .collect()
    }

    /// Adds an element (polygon, path, text, or reference) to the cell.
    pub fn add(&mut self, element: impl Into<Element>) {
        match element.into() {
            Element::Path(path) => self.paths.push(path),
            Element::Polygon(polygon) => self.polygons.push(polygon),
            Element::Box(gds_box) => self.boxes.push(gds_box),
            Element::Reference(reference) => self.references.push(reference),
            Element::Text(text) => self.texts.push(text),
        }
    }

    /// Converts all elements to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            polygons: self
                .polygons
                .into_iter()
                .map(Polygon::to_integer_unit)
                .collect(),
            paths: self.paths.into_iter().map(Path::to_integer_unit).collect(),
            boxes: self
                .boxes
                .into_iter()
                .map(GdsBox::to_integer_unit)
                .collect(),
            texts: self.texts.into_iter().map(Text::to_integer_unit).collect(),
            references: self
                .references
                .into_iter()
                .map(Reference::to_integer_unit)
                .collect(),
            ..self
        }
    }

    /// Converts all elements to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            polygons: self
                .polygons
                .into_iter()
                .map(Polygon::to_float_unit)
                .collect(),
            paths: self.paths.into_iter().map(Path::to_float_unit).collect(),
            boxes: self.boxes.into_iter().map(GdsBox::to_float_unit).collect(),
            texts: self.texts.into_iter().map(Text::to_float_unit).collect(),
            references: self
                .references
                .into_iter()
                .map(Reference::to_float_unit)
                .collect(),
            ..self
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

        for polygon in &self.polygons {
            if tx.send(Element::Polygon(polygon.clone())).is_err() {
                return;
            }
        }

        for path in &self.paths {
            if tx.send(Element::Path(path.clone())).is_err() {
                return;
            }
        }

        for gds_box in &self.boxes {
            if tx.send(Element::Box(gds_box.clone())).is_err() {
                return;
            }
        }

        for text in &self.texts {
            if tx.send(Element::Text(text.clone())).is_err() {
                return;
            }
        }

        for reference in &self.references {
            if reference
                .clone()
                .stream_flatten(Some(depth), library, tx)
                .is_err()
            {
                return;
            }
        }
    }

    /// Returns all elements in this cell, recursively flattening references up to the given depth.
    pub fn get_elements(&self, depth: Option<usize>, library: &Library) -> Vec<Element> {
        let depth = depth.unwrap_or(usize::MAX);
        let mut elements: Vec<Element> = Vec::new();

        for polygon in &self.polygons {
            elements.push(Element::Polygon(polygon.clone()));
        }

        for path in &self.paths {
            elements.push(Element::Path(path.clone()));
        }

        for gds_box in &self.boxes {
            elements.push(Element::Box(gds_box.clone()));
        }

        for text in &self.texts {
            elements.push(Element::Text(text.clone()));
        }

        for reference in &self.references {
            let reference_elements = reference.clone().flatten(Some(depth), library);
            for referenced_element in reference_elements {
                elements.push(referenced_element);
            }
        }

        elements
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cell '{}' with {} polygon(s), {} path(s), {} box(es), {} text(s)",
            self.name,
            self.polygons.len(),
            self.paths.len(),
            self.boxes.len(),
            self.texts.len(),
        )
    }
}

impl Transformable for Cell {
    fn transform_impl(mut self, transformation: &Transformation) -> Self {
        self.polygons = self
            .polygons
            .into_iter()
            .map(|polygon| polygon.transform_impl(transformation))
            .collect();

        self.paths = self
            .paths
            .into_iter()
            .map(|path| path.transform_impl(transformation))
            .collect();

        self.boxes = self
            .boxes
            .into_iter()
            .map(|gds_box| gds_box.transform_impl(transformation))
            .collect();

        self.texts = self
            .texts
            .into_iter()
            .map(|text| text.transform_impl(transformation))
            .collect();

        self.references = self
            .references
            .into_iter()
            .map(|reference| reference.transform_impl(transformation))
            .collect();

        self
    }
}

impl Dimensions for Cell {
    fn bounding_box(&self) -> (Point, Point) {
        let all_points: Vec<Point> = self
            .polygons
            .iter()
            .flat_map(|p| {
                let (min, max) = p.bounding_box();
                vec![min, max]
            })
            .chain(self.paths.iter().flat_map(|p| {
                let (min, max) = p.bounding_box();
                vec![min, max]
            }))
            .chain(self.boxes.iter().flat_map(|b| {
                let (min, max) = b.bounding_box();
                vec![min, max]
            }))
            .chain(self.texts.iter().flat_map(|t| {
                let (min, max) = t.bounding_box();
                vec![min, max]
            }))
            .collect();

        crate::geometry::bounding_box(&all_points)
    }
}

impl Movable for Cell {
    fn move_to(mut self, target: crate::Point) -> Self {
        self.polygons = self
            .polygons
            .into_iter()
            .map(|polygon| polygon.move_to(target))
            .collect();

        self.paths = self
            .paths
            .into_iter()
            .map(|path| path.move_to(target))
            .collect();

        self.boxes = self
            .boxes
            .into_iter()
            .map(|gds_box| gds_box.move_to(target))
            .collect();

        self.texts = self
            .texts
            .into_iter()
            .map(|text| text.move_to(target))
            .collect();

        self.references = self
            .references
            .into_iter()
            .map(|reference| reference.move_to(target))
            .collect();

        self
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

        let path_bufs: Result<Vec<_>, _> = self
            .paths
            .par_iter()
            .map(|p| p.to_gds_impl(database_units))
            .collect();
        for b in path_bufs? {
            buffer.extend_from_slice(&b);
        }

        let polygon_bufs: Result<Vec<_>, _> = self
            .polygons
            .par_iter()
            .map(|p| p.to_gds_impl(database_units))
            .collect();
        for b in polygon_bufs? {
            buffer.extend_from_slice(&b);
        }

        let box_bufs: Result<Vec<_>, _> = self
            .boxes
            .par_iter()
            .map(|b| b.to_gds_impl(database_units))
            .collect();
        for b in box_bufs? {
            buffer.extend_from_slice(&b);
        }

        let text_bufs: Result<Vec<_>, _> = self
            .texts
            .par_iter()
            .map(|t| t.to_gds_impl(database_units))
            .collect();
        for b in text_bufs? {
            buffer.extend_from_slice(&b);
        }

        let ref_bufs: Result<Vec<_>, _> = self
            .references
            .par_iter()
            .map(|r| r.to_gds_impl(database_units))
            .collect();
        for b in ref_bufs? {
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
        assert!(cell.polygons().is_empty());
        assert!(cell.paths().is_empty());
        assert!(cell.boxes().is_empty());
        assert!(cell.texts().is_empty());
        assert!(cell.references().is_empty());
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert_eq!(cell.name, "");
        assert!(cell.polygons.is_empty());
        assert!(cell.paths.is_empty());
        assert!(cell.boxes.is_empty());
        assert!(cell.texts.is_empty());
        assert!(cell.references.is_empty());
    }

    #[test]
    fn test_add_polygon() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::default();

        cell.add(polygon.clone());
        assert_eq!(cell.polygons.len(), 1);
        assert_eq!(cell.polygons[0], polygon);
    }

    #[test]
    fn test_add_path() {
        let mut cell = Cell::new("test_cell");
        let path = Path::default();

        cell.add(path.clone());
        assert_eq!(cell.paths.len(), 1);
        assert_eq!(cell.paths[0], path);
    }

    #[test]
    fn test_add_text() {
        let mut cell = Cell::new("test_cell");
        let text = Text::default();

        cell.add(text.clone());
        assert_eq!(cell.texts.len(), 1);
        assert_eq!(cell.texts[0], text);
    }

    #[test]
    fn test_cell_display() {
        let mut cell = Cell::new("test_cell");
        let polygon = Polygon::default();
        cell.add(polygon);

        insta::assert_snapshot!(cell.to_string(), @"Cell 'test_cell' with 1 polygon(s), 0 path(s), 0 box(es), 0 text(s)");
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

        assert_eq!(converted.polygons().len(), 1);
        assert_eq!(converted.paths().len(), 1);
        assert_eq!(converted.texts().len(), 1);
        for point in converted.polygons()[0].points() {
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

        assert_eq!(converted.polygons().len(), 1);
        for point in converted.polygons()[0].points() {
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
