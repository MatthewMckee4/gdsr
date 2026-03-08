mod gds_format;
pub mod svg;

use std::io::Write;

use chrono::{Datelike, Local, Timelike};
use rayon::prelude::*;

use crate::config::gds_file_types::{GDSDataType, GDSRecord, record_head, record_header};
use crate::elements::text::get_presentation_value;
use crate::error::GdsError;
use crate::{
    Cell, DataType, Element, GdsBox, Instance, Layer, Library, Movable, Node, Path, Point, Polygon,
    Reference, Text, Transformable,
};
use gds_format::{eight_byte_real, write_u16_array_as_big_endian};

pub const MAX_POINTS: usize = 8191;
pub const MAX_LAYER: u16 = 255;
pub const MAX_DATA_TYPE: u16 = 255;
pub const MAX_STRING_LENGTH: usize = 512;
pub const MAX_STRUCTURE_NAME_LENGTH: usize = 32;
pub const MAX_COL_ROW: u32 = 32767;
pub const MIN_POLYGON_POINTS: usize = 4;

/// Trait for customizing GDS file serialization.
///
/// Implement this trait to customize how specific elements are serialized
/// (e.g., filtering layers, transforming during write, or using a different format).
/// All methods have default implementations that delegate to the corresponding
/// free functions in this module, so custom writers only need to override the
/// methods they want to change.
pub trait GdsWriter: Sync {
    /// Serializes an entire library to GDS bytes.
    fn write_library(
        &self,
        library: &Library,
        user_units: f64,
        db_units: f64,
    ) -> Result<Vec<u8>, GdsError> {
        write_library(library, user_units, db_units)
    }

    /// Serializes a single cell to GDS bytes.
    fn write_cell(&self, cell: &Cell, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_cell(cell, db_units)
    }

    /// Serializes a single element to GDS bytes.
    fn write_element(&self, element: &Element, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_element(element, db_units)
    }

    /// Serializes a polygon to GDS bytes.
    fn write_polygon(&self, polygon: &Polygon, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_polygon(polygon, db_units)
    }

    /// Serializes a path to GDS bytes.
    fn write_path(&self, path: &Path, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_path(path, db_units)
    }

    /// Serializes a text element to GDS bytes.
    fn write_text(&self, text: &Text, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_text(text, db_units)
    }

    /// Serializes a reference to GDS bytes.
    fn write_reference(&self, reference: &Reference, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_reference(reference, db_units)
    }

    /// Serializes a box to GDS bytes.
    fn write_box(&self, gds_box: &GdsBox, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_box(gds_box, db_units)
    }

    /// Serializes a node to GDS bytes.
    fn write_node(&self, node: &Node, db_units: f64) -> Result<Vec<u8>, GdsError> {
        write_node(node, db_units)
    }
}

/// Default GDS writer using standard GDSII serialization.
pub struct GdsFileWriter;

impl GdsWriter for GdsFileWriter {}

pub fn write_u16_array(buffer: &mut impl Write, array: &[u16]) -> Result<(), GdsError> {
    Ok(write_u16_array_as_big_endian(buffer, array)?)
}

fn write_float_to_eight_byte_real_to_file(
    buffer: &mut impl Write,
    value: f64,
) -> Result<(), GdsError> {
    let value = eight_byte_real(value);
    Ok(buffer.write_all(&value)?)
}

/// Returns the current timestamp as 12 u16 values (creation + modification) for GDS records.
fn gds_timestamp() -> [u16; 12] {
    let now = Local::now();
    let ts = now.naive_utc();
    [
        ts.year() as u16,
        ts.month() as u16,
        ts.day() as u16,
        ts.hour() as u16,
        ts.minute() as u16,
        ts.second() as u16,
        ts.year() as u16,
        ts.month() as u16,
        ts.day() as u16,
        ts.hour() as u16,
        ts.minute() as u16,
        ts.second() as u16,
    ]
}

/// Returns the `BgnStr` record header with the current timestamp.
fn cell_head_record() -> [u16; 14] {
    let [size, head] = record_header(GDSRecord::BgnStr, GDSDataType::TwoByteSignedInteger, 12);
    let ts = gds_timestamp();
    [
        size, head, ts[0], ts[1], ts[2], ts[3], ts[4], ts[5], ts[6], ts[7], ts[8], ts[9], ts[10],
        ts[11],
    ]
}

/// Writes a single i32 value as a `FourByteSignedInteger` GDS record.
fn write_i32_record(
    buffer: &mut impl Write,
    record: GDSRecord,
    value: u32,
) -> Result<(), GdsError> {
    write_u16_array(
        buffer,
        &record_header(record, GDSDataType::FourByteSignedInteger, 1),
    )?;
    buffer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Returns an `InvalidInput` error if the layer is out of the GDS2 spec range (0-255).
pub fn validate_layer(layer: Layer) -> Result<(), GdsError> {
    if layer.value() > MAX_LAYER {
        return Err(GdsError::ValidationError {
            message: format!(
                "Layer {} exceeds maximum value of {MAX_LAYER}",
                layer.value()
            ),
        });
    }
    Ok(())
}

/// Returns a validation error if the data type is out of the GDS2 spec range (0-255).
pub fn validate_data_type(data_type: DataType) -> Result<(), GdsError> {
    if data_type.value() > MAX_DATA_TYPE {
        return Err(GdsError::ValidationError {
            message: format!(
                "Data type {} exceeds maximum value of {MAX_DATA_TYPE}",
                data_type.value()
            ),
        });
    }
    Ok(())
}

/// Returns a validation error if the string exceeds 512 characters.
fn validate_string_length(s: &str) -> Result<(), GdsError> {
    if s.len() > MAX_STRING_LENGTH {
        return Err(GdsError::ValidationError {
            message: format!(
                "String length {} exceeds maximum of {MAX_STRING_LENGTH} characters",
                s.len()
            ),
        });
    }
    Ok(())
}

/// Returns a validation error if the structure name exceeds 32 characters or contains
/// invalid characters (only alphanumeric, `_`, `?`, `$` are allowed).
fn validate_structure_name(name: &str) -> Result<(), GdsError> {
    if name.len() > MAX_STRUCTURE_NAME_LENGTH {
        return Err(GdsError::ValidationError {
            message: format!(
                "Structure name length {} exceeds maximum of {MAX_STRUCTURE_NAME_LENGTH} characters",
                name.len()
            ),
        });
    }
    if let Some(c) = name
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '?' && *c != '$')
    {
        return Err(GdsError::ValidationError {
            message: format!("Structure name contains invalid character: '{c}'"),
        });
    }
    Ok(())
}

/// Returns a validation error if columns or rows exceed 32767.
fn validate_col_row(columns: u32, rows: u32) -> Result<(), GdsError> {
    if columns > MAX_COL_ROW {
        return Err(GdsError::ValidationError {
            message: format!("Column count {columns} exceeds maximum value of {MAX_COL_ROW}"),
        });
    }
    if rows > MAX_COL_ROW {
        return Err(GdsError::ValidationError {
            message: format!("Row count {rows} exceeds maximum value of {MAX_COL_ROW}"),
        });
    }
    Ok(())
}

/// Validates that a polygon has the correct number of points for GDS serialization.
fn validate_polygon_points(points: &[Point]) -> Result<(), GdsError> {
    if points.len() > MAX_POINTS {
        return Err(GdsError::ValidationError {
            message: format!(
                "Polygon has {} points, which exceeds the maximum of {}",
                points.len(),
                MAX_POINTS
            ),
        });
    }

    if points.len() < MIN_POLYGON_POINTS {
        return Err(GdsError::ValidationError {
            message: format!(
                "Polygon must have at least {MIN_POLYGON_POINTS} points (3 vertices + closing point), got {}",
                points.len()
            ),
        });
    }

    Ok(())
}

fn write_gds_head_to_file(
    library_name: &str,
    user_units: f64,
    db_units: f64,
    buffer: &mut impl Write,
) -> Result<(), GdsError> {
    let [s1, h1] = record_header(GDSRecord::Header, GDSDataType::TwoByteSignedInteger, 1);
    let [s2, h2] = record_header(GDSRecord::BgnLib, GDSDataType::TwoByteSignedInteger, 12);
    let ts = gds_timestamp();
    let head_start = [
        s1, h1, 0x0258, s2, h2, ts[0], ts[1], ts[2], ts[3], ts[4], ts[5], ts[6], ts[7], ts[8],
        ts[9], ts[10], ts[11],
    ];

    write_u16_array(buffer, &head_start)?;
    write_string_with_record_to_file(buffer, GDSRecord::LibName, library_name)?;
    write_u16_array(
        buffer,
        &record_header(GDSRecord::Units, GDSDataType::EightByteReal, 2),
    )?;
    write_float_to_eight_byte_real_to_file(buffer, user_units)?;
    write_float_to_eight_byte_real_to_file(buffer, db_units)
}

fn write_gds_tail_to_file(buffer: &mut impl Write) -> Result<(), GdsError> {
    write_u16_array(
        buffer,
        &record_header(GDSRecord::EndLib, GDSDataType::NoData, 1),
    )
}

fn write_points_to_file(
    buffer: &mut impl Write,
    points: &[Point],
    database_units: f64,
) -> Result<(), GdsError> {
    let num_points = points.len().min(MAX_POINTS);

    let record_size = GDSDataType::FourByteSignedInteger.record_size(num_points as u16 * 2);
    let xy_header_buffer = [
        record_size,
        record_head(GDSRecord::XY, GDSDataType::FourByteSignedInteger),
    ];

    write_u16_array(buffer, &xy_header_buffer)?;

    for point in points.iter().take(num_points) {
        let point = point.to_integer_unit();
        let x_real = point.x().absolute_value();
        let y_real = point.y().absolute_value();

        let scaled_x = (x_real / database_units).round() as i32;
        let scaled_y = (y_real / database_units).round() as i32;

        buffer.write_all(&scaled_x.to_be_bytes())?;
        buffer.write_all(&scaled_y.to_be_bytes())?;
    }

    Ok(())
}

fn write_element_tail_to_file(buffer: &mut impl Write) -> Result<(), GdsError> {
    let tail = [
        GDSDataType::NoData.record_size(1),
        record_head(GDSRecord::EndEl, GDSDataType::NoData),
    ];
    write_u16_array(buffer, &tail)
}

fn write_string_with_record_to_file(
    buffer: &mut impl Write,
    record: GDSRecord,
    string: &str,
) -> Result<(), GdsError> {
    let byte_len = string.len();
    let padded_len = byte_len + (byte_len % 2);

    let string_start = [
        (4 + padded_len) as u16,
        record_head(record, GDSDataType::AsciiString),
    ];

    write_u16_array(buffer, &string_start)?;

    buffer.write_all(string.as_bytes())?;
    if byte_len % 2 != 0 {
        buffer.write_all(&[0])?;
    }

    Ok(())
}

fn write_transformation_to_file(
    buffer: &mut impl Write,
    angle: f64,
    magnification: f64,
    x_reflection: bool,
) -> Result<(), GdsError> {
    let transform_applied = angle != 0.0 || magnification != 1.0 || x_reflection;
    if transform_applied {
        let buffer_flags = [
            GDSDataType::BitArray.record_size(1),
            record_head(GDSRecord::STrans, GDSDataType::BitArray),
            if x_reflection { 0x8000 } else { 0x0000 },
        ];

        write_u16_array(buffer, &buffer_flags)?;

        if magnification != 1.0 {
            let buffer_mag = [
                GDSDataType::EightByteReal.record_size(1),
                record_head(GDSRecord::Mag, GDSDataType::EightByteReal),
            ];
            write_u16_array(buffer, &buffer_mag)?;
            write_float_to_eight_byte_real_to_file(buffer, magnification)?;
        }

        if angle != 0.0 {
            let buffer_rot = [
                GDSDataType::EightByteReal.record_size(1),
                record_head(GDSRecord::Angle, GDSDataType::EightByteReal),
            ];
            write_u16_array(buffer, &buffer_rot)?;
            write_float_to_eight_byte_real_to_file(buffer, angle)?;
        }
    }

    Ok(())
}

/// Builds the common element header: record type, layer, and sub-type (DataType/BoxType/NodeType).
fn element_head(
    record: GDSRecord,
    layer: u16,
    type_record: GDSRecord,
    type_value: u16,
) -> [u16; 8] {
    let [s1, h1] = record_header(record, GDSDataType::NoData, 1);
    let [s2, h2] = record_header(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger, 1);
    let [s3, h3] = record_header(type_record, GDSDataType::TwoByteSignedInteger, 1);
    [s1, h1, s2, h2, layer, s3, h3, type_value]
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

    let mut buffer = Vec::new();
    write_u16_array(&mut buffer, &cell_head_record())?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::StrName, cell.name())?;

    let element_bufs: Result<Vec<_>, _> = cell
        .elements()
        .par_iter()
        .map(|e| write_element(e, db_units))
        .collect();
    for b in element_bufs? {
        buffer.extend_from_slice(&b);
    }

    write_u16_array(
        &mut buffer,
        &record_header(GDSRecord::EndStr, GDSDataType::NoData, 1),
    )?;

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
    validate_polygon_points(polygon.points())?;
    validate_layer(polygon.layer())?;
    validate_data_type(polygon.data_type())?;

    let mut buffer = Vec::new();

    let head = element_head(
        GDSRecord::Boundary,
        polygon.layer().value(),
        GDSRecord::DataType,
        polygon.data_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;
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

    let head = element_head(
        GDSRecord::Path,
        path.layer().value(),
        GDSRecord::DataType,
        path.data_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;

    if let Some(path_type) = path.path_type() {
        let [size, head] = record_header(GDSRecord::PathType, GDSDataType::TwoByteSignedInteger, 1);
        write_u16_array(&mut buffer, &[size, head, path_type.value()])?;
    }

    if let Some(width) = path.width() {
        let scaled_width = width.scale_to(db_units);
        let width_value = scaled_width.as_integer_unit().value as u32;
        write_i32_record(&mut buffer, GDSRecord::Width, width_value)?;
    }

    if let Some(begin_ext) = path.begin_extension() {
        let scaled = begin_ext.scale_to(db_units);
        let value = scaled.as_integer_unit().value as u32;
        write_i32_record(&mut buffer, GDSRecord::BgnExtn, value)?;
    }

    if let Some(end_ext) = path.end_extension() {
        let scaled = end_ext.scale_to(db_units);
        let value = scaled.as_integer_unit().value as u32;
        write_i32_record(&mut buffer, GDSRecord::EndExtn, value)?;
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

    let [s1, h1] = record_header(GDSRecord::Text, GDSDataType::NoData, 1);
    let [s2, h2] = record_header(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger, 1);
    let [s3, h3] = record_header(GDSRecord::TextType, GDSDataType::TwoByteSignedInteger, 1);
    let [s4, h4] = record_header(GDSRecord::Presentation, GDSDataType::BitArray, 1);
    let buffer_start = [
        s1,
        h1,
        s2,
        h2,
        text.layer().value(),
        s3,
        h3,
        0,
        s4,
        h4,
        get_presentation_value(
            *text.vertical_presentation(),
            *text.horizontal_presentation(),
        ),
    ];

    write_u16_array(&mut buffer, &buffer_start)?;
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
    let grid = reference.grid();
    validate_col_row(grid.columns(), grid.rows())?;

    let is_single_instance = grid.columns() == 1 && grid.rows() == 1;

    let record = if is_single_instance {
        GDSRecord::SRef
    } else {
        GDSRecord::ARef
    };

    let mut buffer = Vec::new();

    write_u16_array(&mut buffer, &record_header(record, GDSDataType::NoData, 1))?;
    write_string_with_record_to_file(&mut buffer, GDSRecord::SName, cell_name)?;
    write_transformation_to_file(
        &mut buffer,
        grid.angle().to_degrees(),
        grid.magnification(),
        grid.x_reflection(),
    )?;

    if is_single_instance {
        write_points_to_file(&mut buffer, &[grid.origin()], db_units)?;
    } else {
        let [size, head] = record_header(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger, 2);
        write_u16_array(
            &mut buffer,
            &[size, head, grid.columns() as u16, grid.rows() as u16],
        )?;

        let origin = grid
            .origin()
            .rotate_around_point(grid.angle(), &grid.origin());

        let point2 = grid
            .spacing_x()
            .map(|sx| (origin + sx * grid.columns()).rotate_around_point(grid.angle(), &origin))
            .unwrap_or(origin);
        let point3 = grid
            .spacing_y()
            .map(|sy| (origin + sy * grid.rows()).rotate_around_point(grid.angle(), &origin))
            .unwrap_or(origin);

        if grid.spacing_x().is_some() || grid.spacing_y().is_some() {
            write_points_to_file(&mut buffer, &[origin, point2, point3], db_units)?;
        } else {
            write_points_to_file(&mut buffer, &[origin], db_units)?;
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

    let head = element_head(
        GDSRecord::Box,
        gds_box.layer().value(),
        GDSRecord::BoxType,
        gds_box.box_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;
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

    let head = element_head(
        GDSRecord::Node,
        node.layer().value(),
        GDSRecord::NodeType,
        node.node_type().value(),
    );
    write_u16_array(&mut buffer, &head)?;
    write_points_to_file(&mut buffer, node.points(), db_units)?;
    write_element_tail_to_file(&mut buffer)?;

    Ok(buffer)
}
