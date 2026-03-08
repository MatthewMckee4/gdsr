use chrono::{Datelike, Local, Timelike};
use rayon::prelude::*;

use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::traits::ToGds;
use crate::utils::io::{
    validate_structure_name, write_gds_head_to_file, write_gds_tail_to_file,
    write_string_with_record_to_file, write_u16_array_to_file,
};
use crate::{Cell, Element, GdsBox, GdsError, Library, Node, Path, Polygon, Reference, Text};

/// Trait for customizing GDS file serialization.
///
/// All methods have default implementations that produce standard GDSII output.
/// Override individual methods to customize how specific elements are serialized
/// (e.g., filtering layers, transforming during write, or using a different format).
///
/// The call hierarchy is:
/// ```text
/// write_library → write_cell → write_element → write_polygon
///                                             → write_path
///                                             → write_text
///                                             → write_reference
///                                             → write_box
///                                             → write_node
/// ```
pub trait GdsWriter: Sync {
    /// Serializes an entire library to GDS bytes.
    fn write_library(
        &self,
        library: &Library,
        user_units: f64,
        db_units: f64,
    ) -> Result<Vec<u8>, GdsError> {
        let cells: Vec<&Cell> = library.cells().values().collect();
        let cell_buffers: Result<Vec<Vec<u8>>, GdsError> = cells
            .par_iter()
            .map(|cell| self.write_cell(cell, db_units))
            .collect();
        let cell_buffers = cell_buffers?;

        let mut buffer = Vec::new();
        write_gds_head_to_file(library.name(), user_units, db_units, &mut buffer)?;
        for buf in &cell_buffers {
            buffer.extend_from_slice(buf);
        }
        write_gds_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }

    /// Serializes a single cell to GDS bytes.
    fn write_cell(&self, cell: &Cell, db_units: f64) -> Result<Vec<u8>, GdsError> {
        validate_structure_name(cell.name())?;

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
        write_string_with_record_to_file(&mut buffer, GDSRecord::StrName, cell.name())?;

        let element_bufs: Result<Vec<_>, _> = cell
            .elements()
            .par_iter()
            .map(|e| self.write_element(e, db_units))
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

    /// Serializes a single element to GDS bytes, dispatching to the appropriate method.
    fn write_element(&self, element: &Element, db_units: f64) -> Result<Vec<u8>, GdsError> {
        match element {
            Element::Polygon(polygon) => self.write_polygon(polygon, db_units),
            Element::Path(path) => self.write_path(path, db_units),
            Element::Text(text) => self.write_text(text, db_units),
            Element::Reference(reference) => self.write_reference(reference, db_units),
            Element::Box(gds_box) => self.write_box(gds_box, db_units),
            Element::Node(node) => self.write_node(node, db_units),
        }
    }

    /// Serializes a polygon to GDS bytes.
    fn write_polygon(&self, polygon: &Polygon, db_units: f64) -> Result<Vec<u8>, GdsError> {
        polygon.to_gds_impl(db_units)
    }

    /// Serializes a path to GDS bytes.
    fn write_path(&self, path: &Path, db_units: f64) -> Result<Vec<u8>, GdsError> {
        path.to_gds_impl(db_units)
    }

    /// Serializes a text element to GDS bytes.
    fn write_text(&self, text: &Text, db_units: f64) -> Result<Vec<u8>, GdsError> {
        text.to_gds_impl(db_units)
    }

    /// Serializes a reference to GDS bytes.
    fn write_reference(&self, reference: &Reference, db_units: f64) -> Result<Vec<u8>, GdsError> {
        reference.to_gds_impl(db_units)
    }

    /// Serializes a box to GDS bytes.
    fn write_box(&self, gds_box: &GdsBox, db_units: f64) -> Result<Vec<u8>, GdsError> {
        gds_box.to_gds_impl(db_units)
    }

    /// Serializes a node to GDS bytes.
    fn write_node(&self, node: &Node, db_units: f64) -> Result<Vec<u8>, GdsError> {
        node.to_gds_impl(db_units)
    }
}

/// Default GDS writer using standard GDSII serialization.
pub struct GdsFileWriter;

impl GdsWriter for GdsFileWriter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Layer, Point};

    #[test]
    fn write_returns_valid_gds() {
        let mut library = Library::new("test_lib");
        let mut cell = Cell::new("cell");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
                Point::integer(10, 10, 1e-9),
                Point::integer(0, 0, 1e-9),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        library.add_cell(cell);

        let bytes = library.write(&GdsFileWriter, 1e-9, 1e-9).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.gds");
        std::fs::write(&path, &bytes).unwrap();
        let read = Library::read_file(&path, Some(1e-9)).unwrap();

        assert_eq!(library, read);
    }
}
