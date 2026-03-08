use std::io::Write;

use chrono::{Datelike, Local, Timelike};
use rayon::prelude::*;

use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::elements::text::get_presentation_value;
use crate::utils::io::{
    MAX_POINTS, MIN_POLYGON_POINTS, validate_col_row, validate_data_type, validate_layer,
    validate_string_length, validate_structure_name, write_element_tail_to_file,
    write_gds_head_to_file, write_gds_tail_to_file, write_points_to_file,
    write_string_with_record_to_file, write_transformation_to_file, write_u16_array_to_file,
};
use crate::{
    Cell, Element, GdsBox, GdsError, Instance, Library, Movable, Node, Path, Point, Polygon,
    Reference, Text, Transformable,
};

/// Trait for customizing GDS file serialization.
///
/// Implement this trait to customize how specific elements are serialized
/// (e.g., filtering layers, transforming during write, or using a different format).
/// Free functions in this module contain the standard implementations and can be
/// called from custom writers to reuse the default behavior selectively.
pub trait GdsWriter: Sync {
    /// Serializes an entire library to GDS bytes.
    fn write_library(
        &self,
        library: &Library,
        user_units: f64,
        db_units: f64,
    ) -> Result<Vec<u8>, GdsError>;

    /// Serializes a single cell to GDS bytes.
    fn write_cell(&self, cell: &Cell, db_units: f64) -> Result<Vec<u8>, GdsError>;

    /// Serializes a single element to GDS bytes.
    fn write_element(&self, element: &Element, db_units: f64) -> Result<Vec<u8>, GdsError>;

    /// Serializes a polygon to GDS bytes.
    fn write_polygon(&self, polygon: &Polygon, db_units: f64) -> Result<Vec<u8>, GdsError>;

    /// Serializes a path to GDS bytes.
    fn write_path(&self, path: &Path, db_units: f64) -> Result<Vec<u8>, GdsError>;

    /// Serializes a text element to GDS bytes.
    fn write_text(&self, text: &Text, db_units: f64) -> Result<Vec<u8>, GdsError>;

    /// Serializes a reference to GDS bytes.
    fn write_reference(&self, reference: &Reference, db_units: f64) -> Result<Vec<u8>, GdsError>;

    /// Serializes a box to GDS bytes.
    fn write_box(&self, gds_box: &GdsBox, db_units: f64) -> Result<Vec<u8>, GdsError>;

    /// Serializes a node to GDS bytes.
    fn write_node(&self, node: &Node, db_units: f64) -> Result<Vec<u8>, GdsError>;
}

/// Default GDS writer using standard GDSII serialization.
///
/// All methods delegate to the corresponding free functions in this module.
pub struct GdsFileWriter;

impl GdsWriter for GdsFileWriter {
    fn write_library(
        &self,
        library: &Library,
        user_units: f64,
        db_units: f64,
    ) -> Result<Vec<u8>, GdsError> {
        write_library(library, user_units, db_units)
    }

    fn write_cell(&self, cell: &Cell, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_cell(cell, db_units)
    }

    fn write_element(&self, element: &Element, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_element(element, db_units)
    }

    fn write_polygon(&self, polygon: &Polygon, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_polygon(polygon, db_units)
    }

    fn write_path(&self, path: &Path, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_path(path, db_units)
    }

    fn write_text(&self, text: &Text, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_text(text, db_units)
    }

    fn write_reference(&self, reference: &Reference, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_reference(reference, db_units)
    }

    fn write_box(&self, gds_box: &GdsBox, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_box(gds_box, db_units)
    }

    fn write_node(&self, node: &Node, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_node(node, db_units)
    }
}

/// Serializes an entire library to GDS bytes.
pub fn write_library(
    library: &Library,
    user_units: f64,
    db_units: f64,
) -> Result<Vec<u8>, GdsError> {
    let cells: Vec<&Cell> = library.cells().values().collect();
    let cell_buffers: Result<Vec<Vec<u8>>, GdsError> = cells
        .par_iter()
        .map(|cell| write_cell(cell, db_units))
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
pub fn write_cell(cell: &Cell, db_units: f64) -> Result<Vec<u8>, GdsError> {
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
        .map(|e| write_element(e, db_units))
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

/// Serializes a single element to GDS bytes, dispatching to the appropriate function.
pub fn write_element(element: &Element, db_units: f64) -> Result<Vec<u8>, GdsError> {
    match element {
        Element::Polygon(polygon) => write_polygon(polygon, db_units),
        Element::Path(path) => write_path(path, db_units),
        Element::Text(text) => write_text(text, db_units),
        Element::Reference(reference) => write_reference(reference, db_units),
        Element::Box(gds_box) => write_box(gds_box, db_units),
        Element::Node(node) => write_node(node, db_units),
    }
}

/// Serializes a polygon to GDS bytes.
pub fn write_polygon(polygon: &Polygon, db_units: f64) -> Result<Vec<u8>, GdsError> {
    if polygon.points().len() > MAX_POINTS {
        return Err(GdsError::ValidationError {
            message: format!(
                "Polygon has {} points, which exceeds the maximum of {}",
                polygon.points().len(),
                MAX_POINTS
            ),
        });
    }

    validate_layer(polygon.layer())?;
    validate_data_type(polygon.data_type())?;

    if polygon.points().len() < MIN_POLYGON_POINTS {
        return Err(GdsError::ValidationError {
            message: format!(
                "Polygon must have at least {MIN_POLYGON_POINTS} points (3 vertices + closing point), got {}",
                polygon.points().len()
            ),
        });
    }

    let mut buffer = Vec::new();

    let polygon_head = [
        4,
        combine_record_and_data_type(GDSRecord::Boundary, GDSDataType::NoData),
        6,
        combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
        polygon.layer().value(),
        6,
        combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
        polygon.data_type().value(),
    ];

    write_u16_array_to_file(&mut buffer, &polygon_head)?;
    write_points_to_file(&mut buffer, polygon.points(), db_units)?;
    write_element_tail_to_file(&mut buffer)?;

    Ok(buffer)
}

/// Serializes a path to GDS bytes.
pub fn write_path(path: &Path, db_units: f64) -> Result<Vec<u8>, GdsError> {
    validate_layer(path.layer())?;
    validate_data_type(path.data_type())?;

    if path.points().len() < 2 {
        return Err(GdsError::ValidationError {
            message: "Path must have at least 2 points".to_string(),
        });
    }

    let mut buffer = Vec::new();

    let path_head = [
        4,
        combine_record_and_data_type(GDSRecord::Path, GDSDataType::NoData),
        6,
        combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
        path.layer().value(),
        6,
        combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
        path.data_type().value(),
    ];

    write_u16_array_to_file(&mut buffer, &path_head)?;

    if let Some(path_type) = path.path_type() {
        let path_type_head = [
            6,
            combine_record_and_data_type(GDSRecord::PathType, GDSDataType::TwoByteSignedInteger),
            path_type.value(),
        ];
        write_u16_array_to_file(&mut buffer, &path_type_head)?;
    }

    if let Some(width) = path.width() {
        let scaled_width = width.scale_to(db_units);
        let width_value = scaled_width.as_integer_unit().value as u32;
        let width_head = [
            8,
            combine_record_and_data_type(GDSRecord::Width, GDSDataType::FourByteSignedInteger),
        ];
        write_u16_array_to_file(&mut buffer, &width_head)?;
        buffer.write_all(&width_value.to_be_bytes())?;
    }

    if let Some(begin_ext) = path.begin_extension() {
        let scaled = begin_ext.scale_to(db_units);
        let value = scaled.as_integer_unit().value as u32;
        write_u16_array_to_file(
            &mut buffer,
            &[
                8,
                combine_record_and_data_type(
                    GDSRecord::BgnExtn,
                    GDSDataType::FourByteSignedInteger,
                ),
            ],
        )?;
        buffer.write_all(&value.to_be_bytes())?;
    }

    if let Some(end_ext) = path.end_extension() {
        let scaled = end_ext.scale_to(db_units);
        let value = scaled.as_integer_unit().value as u32;
        write_u16_array_to_file(
            &mut buffer,
            &[
                8,
                combine_record_and_data_type(
                    GDSRecord::EndExtn,
                    GDSDataType::FourByteSignedInteger,
                ),
            ],
        )?;
        buffer.write_all(&value.to_be_bytes())?;
    }

    write_points_to_file(&mut buffer, path.points(), db_units)?;
    write_element_tail_to_file(&mut buffer)?;

    Ok(buffer)
}

/// Serializes a text element to GDS bytes.
pub fn write_text(text: &Text, db_units: f64) -> Result<Vec<u8>, GdsError> {
    validate_layer(text.layer())?;
    validate_string_length(text.text())?;

    let mut buffer = Vec::new();

    let buffer_start = [
        4,
        combine_record_and_data_type(GDSRecord::Text, GDSDataType::NoData),
        6,
        combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
        text.layer().value(),
        6,
        combine_record_and_data_type(GDSRecord::TextType, GDSDataType::TwoByteSignedInteger),
        0,
        6,
        combine_record_and_data_type(GDSRecord::Presentation, GDSDataType::BitArray),
        get_presentation_value(
            *text.vertical_presentation(),
            *text.horizontal_presentation(),
        ),
    ];

    write_u16_array_to_file(&mut buffer, &buffer_start)?;
    write_transformation_to_file(
        &mut buffer,
        text.angle(),
        text.magnification(),
        text.x_reflection(),
    )?;
    write_points_to_file(&mut buffer, &[*text.origin()], db_units)?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::String, text.text())?;
    write_element_tail_to_file(&mut buffer)?;

    Ok(buffer)
}

/// Serializes a reference to GDS bytes.
pub fn write_reference(reference: &Reference, db_units: f64) -> Result<Vec<u8>, GdsError> {
    match reference.instance() {
        Instance::Cell(cell_name) => write_reference_cell(reference, db_units, cell_name),
        Instance::Element(element) => {
            write_reference_element(reference, db_units, element.as_ref().as_ref())
        }
    }
}

fn write_reference_element(
    reference: &Reference,
    db_units: f64,
    element: &Element,
) -> Result<Vec<u8>, GdsError> {
    let grid = reference.grid();
    let spacing_x = grid.spacing_x().unwrap_or_default();
    let spacing_y = grid.spacing_y().unwrap_or_default();

    let mut buf = Vec::new();
    for column_index in 0..grid.columns() {
        for row_index in 0..grid.rows() {
            let offset = (spacing_x * column_index) + (spacing_y * row_index);
            let rotated_offset = offset.rotate_around_point(grid.angle(), &Point::default());
            let final_position = grid.origin() + rotated_offset;

            let mut new_element = element.clone();
            if grid.x_reflection() {
                new_element = new_element.reflect(0.0, Point::default());
            }
            new_element = new_element.rotate(grid.angle(), Point::default());
            new_element = new_element.scale(grid.magnification(), Point::default());
            new_element = new_element.move_by(final_position);

            buf.extend_from_slice(&write_element(&new_element, db_units)?);
        }
    }
    Ok(buf)
}

fn write_reference_cell(
    reference: &Reference,
    db_units: f64,
    cell_name: &str,
) -> Result<Vec<u8>, GdsError> {
    validate_col_row(reference.grid().columns(), reference.grid().rows())?;

    let is_single_instance = reference.grid().columns() == 1 && reference.grid().rows() == 1;

    let record = if is_single_instance {
        GDSRecord::SRef
    } else {
        GDSRecord::ARef
    };

    let mut buffer = Vec::new();

    let buffer_start = [4, combine_record_and_data_type(record, GDSDataType::NoData)];
    write_u16_array_to_file(&mut buffer, &buffer_start)?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::SName, cell_name)?;

    let angle_degrees = reference.grid().angle().to_degrees();
    let magnification = reference.grid().magnification();
    let x_reflection = reference.grid().x_reflection();

    write_transformation_to_file(&mut buffer, angle_degrees, magnification, x_reflection)?;

    if is_single_instance {
        let origin = reference.grid().origin();
        write_points_to_file(&mut buffer, &[origin], db_units)?;
    } else {
        let buffer_array = [
            8,
            combine_record_and_data_type(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger),
            reference.grid().columns() as u16,
            reference.grid().rows() as u16,
        ];
        write_u16_array_to_file(&mut buffer, &buffer_array)?;

        let origin = reference
            .grid()
            .origin()
            .rotate_around_point(reference.grid().angle(), &reference.grid().origin());

        match (reference.grid().spacing_x(), reference.grid().spacing_y()) {
            (Some(spacing_x), Some(spacing_y)) => {
                let point2 = (origin + spacing_x * reference.grid().columns())
                    .rotate_around_point(reference.grid().angle(), &origin);
                let point3 = (origin + spacing_y * reference.grid().rows())
                    .rotate_around_point(reference.grid().angle(), &origin);
                write_points_to_file(&mut buffer, &[origin, point2, point3], db_units)?;
            }
            (Some(spacing_x), None) => {
                let point2 = (origin + spacing_x * reference.grid().columns())
                    .rotate_around_point(reference.grid().angle(), &origin);
                write_points_to_file(&mut buffer, &[origin, point2, origin], db_units)?;
            }
            (None, Some(spacing_y)) => {
                let point3 = (origin + spacing_y * reference.grid().rows())
                    .rotate_around_point(reference.grid().angle(), &origin);
                write_points_to_file(&mut buffer, &[origin, origin, point3], db_units)?;
            }
            _ => {
                write_points_to_file(&mut buffer, &[origin], db_units)?;
            }
        }
    }

    write_element_tail_to_file(&mut buffer)?;

    Ok(buffer)
}

/// Serializes a box to GDS bytes.
pub fn write_box(gds_box: &GdsBox, db_units: f64) -> Result<Vec<u8>, GdsError> {
    validate_layer(gds_box.layer())?;
    validate_data_type(gds_box.box_type())?;

    let mut buffer = Vec::new();

    let box_head = [
        4,
        combine_record_and_data_type(GDSRecord::Box, GDSDataType::NoData),
        6,
        combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
        gds_box.layer().value(),
        6,
        combine_record_and_data_type(GDSRecord::BoxType, GDSDataType::TwoByteSignedInteger),
        gds_box.box_type().value(),
    ];

    write_u16_array_to_file(&mut buffer, &box_head)?;
    let points = gds_box.points();
    write_points_to_file(&mut buffer, &points, db_units)?;
    write_element_tail_to_file(&mut buffer)?;

    Ok(buffer)
}

/// Serializes a node to GDS bytes.
pub fn write_node(node: &Node, db_units: f64) -> Result<Vec<u8>, GdsError> {
    validate_layer(node.layer())?;
    validate_data_type(node.node_type())?;

    let mut buffer = Vec::new();

    let node_head = [
        4,
        combine_record_and_data_type(GDSRecord::Node, GDSDataType::NoData),
        6,
        combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
        node.layer().value(),
        6,
        combine_record_and_data_type(GDSRecord::NodeType, GDSDataType::TwoByteSignedInteger),
        node.node_type().value(),
    ];

    write_u16_array_to_file(&mut buffer, &node_head)?;
    write_points_to_file(&mut buffer, node.points(), db_units)?;
    write_element_tail_to_file(&mut buffer)?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, Layer};

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
