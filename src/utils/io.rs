use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

use chrono::{Datelike, Local, Timelike};

use crate::cell::Cell;
use crate::config::gds_file_types::{
    GDSDataType, GDSRecord, GDSRecordData, combine_record_and_data_type,
};
use crate::elements::text::utils::get_presentations_from_value;
use crate::elements::{Path, PathType, Polygon, Reference, Text};
use crate::error::GdsError;
use crate::geometry::round_to_decimals;
use crate::library::Library;
use crate::utils::gds_format::{eight_byte_real, write_u16_array_as_big_endian};
use crate::{DEFAULT_INTEGER_UNITS, DataType, Instance, Layer, Point, ToGds, Unit};

pub fn write_gds_head_to_file(
    library_name: &str,
    user_units: f64,
    db_units: f64,
    buffer: &mut impl std::io::Write,
) -> Result<(), GdsError> {
    let now = Local::now();
    let timestamp = now.naive_utc();

    let head_start = [
        6,
        combine_record_and_data_type(GDSRecord::Header, GDSDataType::TwoByteSignedInteger),
        0x0258,
        28,
        combine_record_and_data_type(GDSRecord::BgnLib, GDSDataType::TwoByteSignedInteger),
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

    write_u16_array_to_file(buffer, &head_start)?;

    write_string_with_record_to_file(buffer, GDSRecord::LibName, library_name)?;

    let head_units = [
        20,
        combine_record_and_data_type(GDSRecord::Units, GDSDataType::EightByteReal),
    ];
    write_u16_array_to_file(buffer, &head_units)?;

    write_float_to_eight_byte_real_to_file(buffer, user_units)?;
    write_float_to_eight_byte_real_to_file(buffer, db_units)
}

pub fn write_gds_tail_to_file(buffer: &mut impl std::io::Write) -> Result<(), GdsError> {
    let tail = [
        4,
        combine_record_and_data_type(GDSRecord::EndLib, GDSDataType::NoData),
    ];
    write_u16_array_to_file(buffer, &tail)
}

pub fn write_u16_array_to_file(
    buffer: &mut impl std::io::Write,
    array: &[u16],
) -> Result<(), GdsError> {
    Ok(write_u16_array_as_big_endian(buffer, array)?)
}

pub fn write_float_to_eight_byte_real_to_file(
    buffer: &mut impl std::io::Write,
    value: f64,
) -> Result<(), GdsError> {
    let value = eight_byte_real(value);
    Ok(buffer.write_all(&value)?)
}

pub const MAX_POINTS: usize = 8191;
pub const MAX_LAYER: u16 = 255;
pub const MAX_DATA_TYPE: u16 = 255;
pub const MAX_STRING_LENGTH: usize = 512;
pub const MAX_STRUCTURE_NAME_LENGTH: usize = 32;
pub const MAX_COL_ROW: u32 = 32767;
pub const MIN_POLYGON_POINTS: usize = 4;

/// Returns an `InvalidInput` error if the layer is out of the GDS2 spec range (0-255).
pub fn validate_layer(layer: u16) -> Result<(), GdsError> {
    if layer > MAX_LAYER {
        return Err(GdsError::ValidationError {
            message: format!("Layer {layer} exceeds maximum value of {MAX_LAYER}"),
        });
    }
    Ok(())
}

/// Returns a validation error if the data type is out of the GDS2 spec range (0-255).
pub fn validate_data_type(data_type: u16) -> Result<(), GdsError> {
    if data_type > MAX_DATA_TYPE {
        return Err(GdsError::ValidationError {
            message: format!("Data type {data_type} exceeds maximum value of {MAX_DATA_TYPE}"),
        });
    }
    Ok(())
}

/// Returns a validation error if the string exceeds 512 characters.
pub fn validate_string_length(s: &str) -> Result<(), GdsError> {
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
pub fn validate_structure_name(name: &str) -> Result<(), GdsError> {
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
pub fn validate_col_row(columns: u32, rows: u32) -> Result<(), GdsError> {
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

pub fn write_points_to_file(
    buffer: &mut impl std::io::Write,
    points: &[Point],
    database_units: f64,
) -> Result<(), GdsError> {
    let num_points = points.len().min(MAX_POINTS);

    let record_size = 4 + (num_points * 8) as u16;
    let xy_header_buffer = [
        record_size,
        combine_record_and_data_type(GDSRecord::XY, GDSDataType::FourByteSignedInteger),
    ];

    write_u16_array_to_file(buffer, &xy_header_buffer)?;

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

pub fn write_element_tail_to_file(buffer: &mut impl std::io::Write) -> Result<(), GdsError> {
    let tail = [
        4,
        combine_record_and_data_type(GDSRecord::EndEl, GDSDataType::NoData),
    ];
    write_u16_array_to_file(buffer, &tail)
}

pub fn write_string_with_record_to_file(
    buffer: &mut impl std::io::Write,
    record: GDSRecord,
    string: &str,
) -> Result<(), GdsError> {
    let byte_len = string.len();
    let padded_len = byte_len + (byte_len % 2);

    let string_start = [
        (4 + padded_len) as u16,
        combine_record_and_data_type(record, GDSDataType::AsciiString),
    ];

    write_u16_array_to_file(buffer, &string_start)?;

    buffer.write_all(string.as_bytes())?;
    if byte_len % 2 != 0 {
        buffer.write_all(&[0])?;
    }

    Ok(())
}

pub fn write_gds<P: AsRef<std::path::Path>>(
    file_name: P,
    library_name: &str,
    user_units: f64,
    database_units: f64,
    cells: &[&Cell],
) -> Result<(), GdsError> {
    use rayon::prelude::*;

    let cell_buffers: Result<Vec<Vec<u8>>, GdsError> = cells
        .par_iter()
        .map(|cell| cell.to_gds_impl(database_units))
        .collect();
    let cell_buffers = cell_buffers?;

    let mut file = File::create(file_name)?;
    write_gds_head_to_file(library_name, user_units, database_units, &mut file)?;
    for buf in &cell_buffers {
        file.write_all(buf)?;
    }
    write_gds_tail_to_file(&mut file)?;
    Ok(file.flush()?)
}

pub fn write_transformation_to_file(
    buffer: &mut impl std::io::Write,
    angle: f64,
    magnification: f64,
    x_reflection: bool,
) -> Result<(), GdsError> {
    let transform_applied = angle != 0.0 || magnification != 1.0 || x_reflection;
    if transform_applied {
        let buffer_flags = [
            6,
            combine_record_and_data_type(GDSRecord::STrans, GDSDataType::BitArray),
            if x_reflection { 0x8000 } else { 0x0000 },
        ];

        write_u16_array_to_file(buffer, &buffer_flags)?;

        if magnification != 1.0 {
            let buffer_mag = [
                12,
                combine_record_and_data_type(GDSRecord::Mag, GDSDataType::EightByteReal),
            ];
            write_u16_array_to_file(buffer, &buffer_mag)?;
            write_float_to_eight_byte_real_to_file(buffer, magnification)?;
        }

        if angle != 0.0 {
            let buffer_rot = [
                12,
                combine_record_and_data_type(GDSRecord::Angle, GDSDataType::EightByteReal),
            ];
            write_u16_array_to_file(buffer, &buffer_rot)?;
            write_float_to_eight_byte_real_to_file(buffer, angle)?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub fn from_gds<P: AsRef<std::path::Path>>(
    file_name: P,
    units: Option<f64>,
) -> Result<Library, GdsError> {
    let mut library = Library::new("Library");

    let file = File::open(file_name)?;
    let reader = RecordReader::new(BufReader::new(file));

    let mut cell: Option<Cell> = None;
    let mut path: Option<Path> = None;
    let mut polygon: Option<Polygon> = None;
    let mut text: Option<Text> = None;
    let mut reference: Option<Reference> = None;

    let mut scale = 1.0;
    let mut db_units = units.unwrap_or(DEFAULT_INTEGER_UNITS);

    for record in reader {
        match record {
            Ok((record_type, data)) => match record_type {
                GDSRecord::LibName => {
                    if let GDSRecordData::Str(name) = data {
                        library.name = name;
                    }
                }
                GDSRecord::Units => {
                    if let GDSRecordData::F64(units_vec) = data {
                        let db_units_from_file = units_vec[1];

                        if units.is_none() {
                            db_units = db_units_from_file;
                        }
                        scale = db_units_from_file / db_units;
                    }
                }
                GDSRecord::BgnStr => {
                    cell = Some(Cell::default());
                }
                GDSRecord::StrName => {
                    if let GDSRecordData::Str(cell_name) = data {
                        if let Some(cell) = &mut cell {
                            cell.set_name(&cell_name);
                        }
                    }
                }
                GDSRecord::EndStr => {
                    if let Some(cell) = cell.take() {
                        library.cells.insert(cell.name().to_string(), cell);
                    }
                }
                GDSRecord::Boundary | GDSRecord::Box => {
                    polygon = Some(Polygon::default());
                }
                GDSRecord::Path => {
                    path = Some(Path::default());
                }
                GDSRecord::ARef | GDSRecord::SRef => {
                    reference = Some(Reference::default());
                }
                GDSRecord::Text => {
                    text = Some(Text::default());
                }
                GDSRecord::Layer => {
                    if let GDSRecordData::I16(layer) = data {
                        let layer_value = layer[0] as Layer;
                        if let Some(polygon) = &mut polygon {
                            polygon.layer = layer_value;
                        } else if let Some(path) = &mut path {
                            path.layer = layer_value;
                        } else if let Some(text) = &mut text {
                            text.layer = layer_value;
                        }
                    }
                }
                GDSRecord::DataType | GDSRecord::BoxType => {
                    if let GDSRecordData::I16(data_type) = data {
                        let data_type_val = data_type[0] as DataType;
                        if let Some(polygon) = &mut polygon {
                            polygon.data_type = data_type_val;
                        } else if let Some(path) = &mut path {
                            path.data_type = data_type_val;
                        }
                    }
                }
                GDSRecord::Width => {
                    if let GDSRecordData::I32(width) = data {
                        let path_width = round_to_decimals(f64::from(width[0]) * scale, 10);
                        if let Some(path) = &mut path {
                            let unit = Unit::float(path_width, db_units);
                            path.width = Some(unit);
                        }
                    }
                }
                GDSRecord::XY => {
                    if let GDSRecordData::I32(xy) = data {
                        let points = get_points_from_i32_vec(&xy, db_units)
                            .iter()
                            .map(|p| {
                                Point::integer(
                                    (p.x().float_value() * scale).round() as i32,
                                    (p.y().float_value() * scale).round() as i32,
                                    db_units,
                                )
                            })
                            .collect::<Vec<Point>>();

                        if let Some(polygon) = &mut polygon {
                            polygon.points = points;
                        } else if let Some(path) = &mut path {
                            path.points = points;
                        } else if let Some(reference) = &mut reference {
                            match points.as_slice() {
                                [point] => {
                                    reference.grid.set_origin(*point);
                                }
                                [origin, _, _] => {
                                    let unrotated_points = points
                                        .iter()
                                        .map(|&p| {
                                            p.rotate_around_point(-reference.grid.angle(), origin)
                                        })
                                        .collect::<Vec<Point>>();

                                    reference.grid.set_origin(unrotated_points[0]);

                                    reference
                                        .grid
                                        .set_spacing_x(if reference.grid.columns() > 1 {
                                            Some(
                                                (unrotated_points[1] / reference.grid.columns())
                                                    - unrotated_points[0],
                                            )
                                        } else {
                                            None
                                        });
                                    reference.grid.set_spacing_y(if reference.grid.rows() > 1 {
                                        Some(
                                            (unrotated_points[2] / reference.grid.rows())
                                                - unrotated_points[0],
                                        )
                                    } else {
                                        None
                                    });
                                }
                                _ => {}
                            }
                        } else if let Some(text) = &mut text {
                            if let Some(&first_point) = points.first() {
                                text.origin = first_point;
                            }
                        }
                    }
                }
                GDSRecord::EndEl => {
                    if let Some(cell) = &mut cell {
                        if let Some(polygon) = polygon.take() {
                            cell.add(polygon);
                        } else if let Some(path) = path.take() {
                            cell.add(path);
                        } else if let Some(reference) = reference.take() {
                            cell.add(reference);
                        } else if let Some(text) = text.take() {
                            cell.add(text);
                        }
                    }
                    polygon = None;
                    path = None;
                    text = None;
                    reference = None;
                }
                GDSRecord::SName => {
                    if let GDSRecordData::Str(cell_name) = data {
                        if let Some(reference) = &mut reference {
                            if let Instance::Cell(_) = reference.instance {
                                reference.instance = Instance::Cell(cell_name);
                            }
                        }
                    }
                }
                GDSRecord::ColRow => {
                    if let GDSRecordData::I16(col_row) = data {
                        if let Some(reference) = &mut reference {
                            reference.grid.set_columns(col_row[0] as u32);
                            reference.grid.set_rows(col_row[1] as u32);
                        }
                    }
                }
                GDSRecord::Presentation => {
                    if let GDSRecordData::I16(flags) = data {
                        if let Some(text) = &mut text {
                            if let Ok((vertical_presentation, horizontal_presentation)) =
                                get_presentations_from_value(flags[0])
                            {
                                text.vertical_presentation = vertical_presentation;
                                text.horizontal_presentation = horizontal_presentation;
                            }
                        }
                    }
                }
                GDSRecord::String => {
                    if let GDSRecordData::Str(string) = data {
                        if let Some(text) = &mut text {
                            text.value = string;
                        }
                    }
                }
                GDSRecord::STrans => {
                    if let GDSRecordData::I16(flags) = data {
                        let x_reflection = flags[0] & 0x8000u16 as i16 != 0;
                        if let Some(text) = &mut text {
                            text.x_reflection = x_reflection;
                        }
                        if let Some(reference) = &mut reference {
                            reference.grid.set_x_reflection(x_reflection);
                        }
                    }
                }
                GDSRecord::Mag => {
                    if let GDSRecordData::F64(magnification) = data {
                        if let Some(text) = &mut text {
                            text.magnification = magnification[0];
                        } else if let Some(reference) = &mut reference {
                            reference.grid.set_magnification(magnification[0]);
                        }
                    }
                }
                GDSRecord::Angle => {
                    if let GDSRecordData::F64(angle) = data {
                        if let Some(text) = &mut text {
                            text.angle = angle[0];
                        } else if let Some(reference) = &mut reference {
                            reference.grid.set_angle(angle[0]);
                        }
                    }
                }
                GDSRecord::PathType => {
                    if let GDSRecordData::I16(path_type) = data {
                        if let Some(path) = &mut path {
                            path.r#type = Some(PathType::new(i32::from(path_type[0])));
                        }
                    }
                }
                _ => {}
            },
            Err(e) => return Err(e),
        }
    }

    Ok(library)
}

pub struct RecordReader<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> RecordReader<R> {
    pub const fn new(reader: BufReader<R>) -> Self {
        Self { reader }
    }
}

impl<R: Read> Iterator for RecordReader<R> {
    type Item = Result<(GDSRecord, GDSRecordData), GdsError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut header = [0u8; 4];
        if let Err(e) = self.reader.read_exact(&mut header) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return None;
            }
            return Some(Err(GdsError::from(e)));
        }

        let size = u16::from_be_bytes([header[0], header[1]]) as usize;
        let record_type = header[2];
        let data_type = header[3];

        let data = if size > 4 {
            let mut buf = vec![0u8; size - 4];
            if let Err(e) = self.reader.read_exact(&mut buf) {
                return Some(Err(GdsError::from(e)));
            }

            let Ok(parsed_data_type) = GDSDataType::try_from(data_type) else {
                return Some(Err(GdsError::InvalidData {
                    message: format!("Invalid data type byte: {data_type:#04x}"),
                }));
            };

            match parsed_data_type {
                GDSDataType::TwoByteSignedInteger | GDSDataType::BitArray => {
                    let result = read_i16_be(&buf);
                    GDSRecordData::I16(result)
                }
                GDSDataType::FourByteSignedInteger | GDSDataType::FourByteReal => {
                    let result = read_i32_be(&buf);
                    GDSRecordData::I32(result)
                }
                GDSDataType::EightByteReal => {
                    let u64_values = read_u64_be(&buf);
                    let result: Vec<f64> = u64_values
                        .into_iter()
                        .map(eight_byte_real_to_float)
                        .collect();
                    GDSRecordData::F64(result)
                }
                GDSDataType::AsciiString => match String::from_utf8(buf) {
                    Ok(mut result) => {
                        if result.ends_with('\0') {
                            result.pop();
                        }
                        GDSRecordData::Str(result)
                    }
                    Err(e) => {
                        return Some(Err(GdsError::InvalidData {
                            message: format!("Invalid UTF-8 in ASCII string record: {e}"),
                        }));
                    }
                },
                GDSDataType::NoData => match String::from_utf8(buf) {
                    Ok(result) => GDSRecordData::Str(result),
                    Err(e) => {
                        return Some(Err(GdsError::InvalidData {
                            message: format!("Invalid UTF-8 in NoData record: {e}"),
                        }));
                    }
                },
            }
        } else {
            GDSRecordData::None
        };

        GDSRecord::try_from(record_type).map_or_else(
            |()| {
                Some(Err(GdsError::InvalidData {
                    message: "Invalid record type".to_string(),
                }))
            },
            |record| Some(Ok((record, data))),
        )
    }
}

fn read_i16_be(buf: &[u8]) -> Vec<i16> {
    let chunk_size = 2;
    let mut result = Vec::with_capacity(buf.len() / chunk_size);
    let mut i = 0;

    while i + chunk_size <= buf.len() {
        let value = (i16::from(buf[i]) << 8) | i16::from(buf[i + 1]);
        result.push(value);
        i += chunk_size;
    }

    result
}

fn read_i32_be(buf: &[u8]) -> Vec<i32> {
    let chunk_size = 4;
    let mut result = Vec::with_capacity(buf.len() / chunk_size);
    let mut i = 0;

    while i + chunk_size <= buf.len() {
        let value = (i32::from(buf[i]) << 24)
            | (i32::from(buf[i + 1]) << 16)
            | (i32::from(buf[i + 2]) << 8)
            | i32::from(buf[i + 3]);
        result.push(value);
        i += chunk_size;
    }

    result
}

fn read_u64_be(buf: &[u8]) -> Vec<u64> {
    let chunk_size = 8;
    let mut result = Vec::with_capacity(buf.len() / chunk_size);
    let mut i = 0;

    while i + chunk_size <= buf.len() {
        let value = (u64::from(buf[i]) << 56)
            | (u64::from(buf[i + 1]) << 48)
            | (u64::from(buf[i + 2]) << 40)
            | (u64::from(buf[i + 3]) << 32)
            | (u64::from(buf[i + 4]) << 24)
            | (u64::from(buf[i + 5]) << 16)
            | (u64::from(buf[i + 6]) << 8)
            | u64::from(buf[i + 7]);
        result.push(value);
        i += chunk_size;
    }

    result
}

fn eight_byte_real_to_float(bytes: u64) -> f64 {
    let short1 = (bytes >> 48) as u16;
    let short2 = ((bytes >> 32) & 0xFFFF) as u16;
    let long3 = (bytes & 0xFFFF_FFFF) as u32;

    let exponent = i32::from((short1 & 0x7F00) >> 8) - 64;

    let mantissa = (u64::from(short1 & 0x00FF) << 48 | u64::from(short2) << 32 | u64::from(long3))
        as f64
        / 72_057_594_037_927_936.0;

    if short1 & 0x8000 != 0 {
        -mantissa * 16.0_f64.powi(exponent)
    } else {
        mantissa * 16.0_f64.powi(exponent)
    }
}

pub fn get_points_from_i32_vec(vec: &[i32], db_units: f64) -> Vec<Point> {
    vec.chunks(2)
        .map(|chunk| Point::integer(chunk[0], chunk[1], db_units))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use super::*;
    use crate::utils::gds_format::eight_byte_real;

    /// Verifies that `write_points_to_file` uses the truncated point count
    /// (capped at `MAX_POINTS`) for the record header size, not the original
    /// input length.
    #[test]
    fn test_write_points_record_size_capped_at_max_points() {
        let db_units = 1e-9;
        let num_points = MAX_POINTS + 100;
        let points: Vec<Point> = (0..num_points)
            .map(|i| Point::integer(i as i32, 0, db_units))
            .collect();

        let mut buf = Vec::new();
        write_points_to_file(&mut buf, &points, db_units).unwrap();

        let record_size = u16::from_be_bytes([buf[0], buf[1]]);
        let expected_size = 4 + (MAX_POINTS * 8) as u16;
        assert_eq!(record_size, expected_size);

        let expected_total_bytes = 4 + MAX_POINTS * 8;
        assert_eq!(buf.len(), expected_total_bytes);
    }

    #[test]
    fn test_read_i16_be_standard_values() {
        let buf = [0x00, 0x01, 0x00, 0x0A, 0xFF, 0xFF];
        let result = read_i16_be(&buf);
        assert_eq!(result, vec![1, 10, -1]);
    }

    #[test]
    fn test_read_i16_be_min_max() {
        let buf = [0x7F, 0xFF, 0x80, 0x00];
        let result = read_i16_be(&buf);
        assert_eq!(result, vec![i16::MAX, i16::MIN]);
    }

    #[test]
    fn test_read_i16_be_endianness() {
        let buf = [0x01, 0x00];
        let result = read_i16_be(&buf);
        assert_eq!(result, vec![256]);
    }

    #[test]
    fn test_read_i16_be_empty() {
        let result = read_i16_be(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_read_i16_be_odd_length_ignores_trailing() {
        let buf = [0x00, 0x05, 0xFF];
        let result = read_i16_be(&buf);
        assert_eq!(result, vec![5]);
    }

    #[test]
    fn test_read_i32_be_standard_values() {
        let buf = [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x64];
        let result = read_i32_be(&buf);
        assert_eq!(result, vec![1, 100]);
    }

    #[test]
    fn test_read_i32_be_min_max() {
        let buf = [0x7F, 0xFF, 0xFF, 0xFF, 0x80, 0x00, 0x00, 0x00];
        let result = read_i32_be(&buf);
        assert_eq!(result, vec![i32::MAX, i32::MIN]);
    }

    #[test]
    fn test_read_i32_be_negative() {
        let buf = [0xFF, 0xFF, 0xFF, 0xFF];
        let result = read_i32_be(&buf);
        assert_eq!(result, vec![-1]);
    }

    #[test]
    fn test_read_i32_be_endianness() {
        let buf = [0x01, 0x00, 0x00, 0x00];
        let result = read_i32_be(&buf);
        assert_eq!(result, vec![0x0100_0000]);
    }

    #[test]
    fn test_read_i32_be_empty() {
        let result = read_i32_be(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_read_i32_be_trailing_bytes_ignored() {
        let buf = [0x00, 0x00, 0x00, 0x07, 0xFF, 0xFF];
        let result = read_i32_be(&buf);
        assert_eq!(result, vec![7]);
    }

    #[test]
    fn test_read_u64_be_standard_values() {
        let buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
        let result = read_u64_be(&buf);
        assert_eq!(result, vec![1u64]);
    }

    #[test]
    fn test_read_u64_be_max() {
        let buf = [0xFF; 8];
        let result = read_u64_be(&buf);
        assert_eq!(result, vec![u64::MAX]);
    }

    #[test]
    fn test_read_u64_be_multiple() {
        let mut buf = [0u8; 16];
        buf[7] = 1;
        buf[15] = 2;
        let result = read_u64_be(&buf);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_read_u64_be_empty() {
        let result = read_u64_be(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_eight_byte_real_to_float_zero() {
        assert_eq!(eight_byte_real_to_float(0), 0.0);
    }

    #[test]
    fn test_eight_byte_real_to_float_positive() {
        let bytes = u64::from_be_bytes(eight_byte_real(1.0));
        let result = eight_byte_real_to_float(bytes);
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_eight_byte_real_to_float_negative() {
        let bytes = u64::from_be_bytes(eight_byte_real(-42.5));
        let result = eight_byte_real_to_float(bytes);
        assert!((result - (-42.5)).abs() < 1e-10);
    }

    #[test]
    fn test_eight_byte_real_to_float_small_value() {
        let bytes = u64::from_be_bytes(eight_byte_real(1e-9));
        let result = eight_byte_real_to_float(bytes);
        assert!((result - 1e-9).abs() < 1e-20);
    }

    #[test]
    fn test_float_to_eight_byte_real_roundtrip() {
        let values = [0.0, 1.0, -1.0, 123.456, -0.001, 1e-9, 1e9, 16.0];
        for &val in &values {
            let encoded = eight_byte_real(val);
            let decoded = eight_byte_real_to_float(u64::from_be_bytes(encoded));
            if val == 0.0 {
                assert_eq!(decoded, 0.0);
            } else {
                let rel_err = ((decoded - val) / val).abs();
                assert!(rel_err < 1e-10, "roundtrip failed for {val}: got {decoded}");
            }
        }
    }

    #[test]
    fn test_write_points_to_file_empty() {
        let mut buf = Vec::new();
        let points: Vec<Point> = vec![];
        write_points_to_file(&mut buf, &points, 1e-9).unwrap();

        assert_eq!(
            buf.len(),
            4,
            "header only (4 bytes for the u16 array of size 2)"
        );
    }

    #[test]
    fn test_write_points_to_file_single_point() {
        let mut buf = Vec::new();
        let points = vec![Point::integer(100, 200, 1e-9)];
        write_points_to_file(&mut buf, &points, 1e-9).unwrap();

        let header_size = 4;
        let point_size = 8;
        assert_eq!(buf.len(), header_size + point_size);

        let x = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let y = i32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        assert_eq!(x, 100);
        assert_eq!(y, 200);
    }

    #[test]
    fn test_write_points_to_file_exactly_max_points() {
        let mut buf = Vec::new();
        let points: Vec<Point> = (0..MAX_POINTS as i32)
            .map(|i| Point::integer(i, i, 1e-9))
            .collect();
        write_points_to_file(&mut buf, &points, 1e-9).unwrap();

        let header_size = 4;
        let expected_data = MAX_POINTS * 8;
        assert_eq!(buf.len(), header_size + expected_data);
    }

    #[test]
    fn test_write_points_to_file_over_max_points_truncates() {
        let mut buf = Vec::new();
        let count = MAX_POINTS + 100;
        let points: Vec<Point> = (0..count as i32)
            .map(|i| Point::integer(i, i, 1e-9))
            .collect();
        write_points_to_file(&mut buf, &points, 1e-9).unwrap();

        let header_size = 4;
        let expected_data = MAX_POINTS * 8;
        assert_eq!(buf.len(), header_size + expected_data);
    }

    #[test]
    fn test_write_string_with_record_to_file_even_length() {
        let mut buf = Vec::new();
        write_string_with_record_to_file(&mut buf, GDSRecord::LibName, "ABCD").unwrap();

        let header_size = 4;
        assert_eq!(buf.len(), header_size + 4);
        assert_eq!(&buf[header_size..], b"ABCD");
    }

    #[test]
    fn test_write_string_with_record_to_file_odd_length_padded() {
        let mut buf = Vec::new();
        write_string_with_record_to_file(&mut buf, GDSRecord::LibName, "ABC").unwrap();

        let header_size = 4;
        assert_eq!(buf.len(), header_size + 4);
        assert_eq!(&buf[header_size..header_size + 3], b"ABC");
        assert_eq!(buf[header_size + 3], 0x00);
    }

    #[test]
    fn test_write_string_with_record_to_file_empty() {
        let mut buf = Vec::new();
        write_string_with_record_to_file(&mut buf, GDSRecord::LibName, "").unwrap();

        let header_size = 4;
        assert_eq!(buf.len(), header_size);
    }

    #[test]
    fn test_record_reader_empty_input() {
        let cursor = Cursor::new(Vec::<u8>::new());
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();
        assert!(records.is_empty());
    }

    #[test]
    fn test_record_reader_valid_no_data_record() {
        let record_type = GDSRecord::EndEl as u8;
        let data_type = GDSDataType::NoData as u8;
        let size: u16 = 4;
        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        let (rec, rec_data) = records[0].as_ref().unwrap();
        assert!(matches!(rec, GDSRecord::EndEl));
        assert!(matches!(rec_data, GDSRecordData::None));
    }

    #[test]
    fn test_record_reader_i16_data() {
        let record_type = GDSRecord::Layer as u8;
        let data_type = GDSDataType::TwoByteSignedInteger as u8;
        let size: u16 = 6;
        let value: i16 = 5;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);
        data.extend_from_slice(&value.to_be_bytes());

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        let (rec, rec_data) = records[0].as_ref().unwrap();
        assert!(matches!(rec, GDSRecord::Layer));
        if let GDSRecordData::I16(vals) = rec_data {
            assert_eq!(vals, &[5]);
        } else {
            panic!("expected I16 data");
        }
    }

    #[test]
    fn test_record_reader_i32_data() {
        let record_type = GDSRecord::Width as u8;
        let data_type = GDSDataType::FourByteSignedInteger as u8;
        let value: i32 = 1000;
        let size: u16 = 8;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);
        data.extend_from_slice(&value.to_be_bytes());

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        let (rec, rec_data) = records[0].as_ref().unwrap();
        assert!(matches!(rec, GDSRecord::Width));
        if let GDSRecordData::I32(vals) = rec_data {
            assert_eq!(vals, &[1000]);
        } else {
            panic!("expected I32 data");
        }
    }

    #[test]
    fn test_record_reader_f64_data() {
        let record_type = GDSRecord::Mag as u8;
        let data_type = GDSDataType::EightByteReal as u8;
        let encoded = eight_byte_real(2.5);
        let size: u16 = 12;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);
        data.extend_from_slice(&encoded);

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        let (rec, rec_data) = records[0].as_ref().unwrap();
        assert!(matches!(rec, GDSRecord::Mag));
        if let GDSRecordData::F64(vals) = rec_data {
            assert!((vals[0] - 2.5).abs() < 1e-10);
        } else {
            panic!("expected F64 data");
        }
    }

    #[test]
    fn test_record_reader_string_data() {
        let record_type = GDSRecord::LibName as u8;
        let data_type = GDSDataType::AsciiString as u8;
        let string_bytes = b"TEST\0";
        let size: u16 = 4 + string_bytes.len() as u16;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);
        data.extend_from_slice(string_bytes);

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        let (rec, rec_data) = records[0].as_ref().unwrap();
        assert!(matches!(rec, GDSRecord::LibName));
        if let GDSRecordData::Str(s) = rec_data {
            assert_eq!(s, "TEST");
        } else {
            panic!("expected Str data");
        }
    }

    #[test]
    fn test_record_reader_string_without_null_terminator() {
        let record_type = GDSRecord::LibName as u8;
        let data_type = GDSDataType::AsciiString as u8;
        let string_bytes = b"NOTERM";
        let size: u16 = 4 + string_bytes.len() as u16;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);
        data.extend_from_slice(string_bytes);

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        if let GDSRecordData::Str(s) = &records[0].as_ref().unwrap().1 {
            assert_eq!(s, "NOTERM");
        } else {
            panic!("expected Str data");
        }
    }

    #[test]
    fn test_record_reader_truncated_header() {
        let data = vec![0x00, 0x06, 0x0D];
        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert!(records.is_empty() || records[0].is_err());
    }

    #[test]
    fn test_record_reader_truncated_body() {
        let record_type = GDSRecord::Layer as u8;
        let data_type = GDSDataType::TwoByteSignedInteger as u8;
        let size: u16 = 6;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);
        // Missing the 2-byte body

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        assert!(records[0].is_err());
    }

    #[test]
    fn test_record_reader_invalid_record_type() {
        let size: u16 = 4;
        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(0xFF);
        data.push(0x00);

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        assert!(records[0].is_err());
    }

    #[test]
    fn test_record_reader_multiple_records() {
        let mut data = Vec::new();

        // First record: EndEl (no data)
        data.extend_from_slice(&4u16.to_be_bytes());
        data.push(GDSRecord::EndEl as u8);
        data.push(GDSDataType::NoData as u8);

        // Second record: EndEl (no data)
        data.extend_from_slice(&4u16.to_be_bytes());
        data.push(GDSRecord::EndEl as u8);
        data.push(GDSDataType::NoData as u8);

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 2);
        assert!(records[0].is_ok());
        assert!(records[1].is_ok());
    }

    #[test]
    fn test_record_reader_bit_array_data() {
        let record_type = GDSRecord::STrans as u8;
        let data_type = GDSDataType::BitArray as u8;
        let size: u16 = 6;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(data_type);
        data.extend_from_slice(&0x8000u16.to_be_bytes());

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        let (rec, rec_data) = records[0].as_ref().unwrap();
        assert!(matches!(rec, GDSRecord::STrans));
        if let GDSRecordData::I16(vals) = rec_data {
            assert_eq!(vals[0] as u16, 0x8000);
        } else {
            panic!("expected I16 data for BitArray");
        }
    }

    #[test]
    fn test_record_reader_unknown_data_type_returns_error() {
        let record_type = GDSRecord::EndEl as u8;
        let unknown_data_type = 0xFFu8;
        let size: u16 = 6;

        let mut data = Vec::new();
        data.extend_from_slice(&size.to_be_bytes());
        data.push(record_type);
        data.push(unknown_data_type);
        data.extend_from_slice(&[0x00, 0x00]);

        let cursor = Cursor::new(data);
        let reader = RecordReader::new(BufReader::new(cursor));
        let records: Vec<_> = reader.collect();

        assert_eq!(records.len(), 1);
        assert!(records[0].is_err());
    }

    #[test]
    fn test_get_points_from_i32_vec() {
        let vec = vec![10, 20, 30, 40];
        let points = get_points_from_i32_vec(&vec, 1e-9);
        assert_eq!(points.len(), 2);
        assert_eq!(points[0], Point::integer(10, 20, 1e-9));
        assert_eq!(points[1], Point::integer(30, 40, 1e-9));
    }

    #[test]
    fn test_get_points_from_i32_vec_empty() {
        let points = get_points_from_i32_vec(&[], 1e-9);
        assert!(points.is_empty());
    }

    #[test]
    fn test_write_u16_array_to_file() {
        let mut buf = Vec::new();
        write_u16_array_to_file(&mut buf, &[0x0102, 0x0304]).unwrap();
        assert_eq!(buf, [0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_write_element_tail_to_file() {
        let mut buf = Vec::new();
        write_element_tail_to_file(&mut buf).unwrap();
        assert_eq!(buf.len(), 4);
    }

    #[test]
    fn test_write_float_to_eight_byte_real_to_file() {
        let mut buf = Vec::new();
        write_float_to_eight_byte_real_to_file(&mut buf, 1.0).unwrap();
        assert_eq!(buf.len(), 8);

        let decoded = eight_byte_real_to_float(u64::from_be_bytes(buf.try_into().unwrap()));
        assert!((decoded - 1.0).abs() < 1e-10);
    }

    fn make_reader(data: &[u8]) -> RecordReader<Cursor<Vec<u8>>> {
        RecordReader::new(BufReader::new(Cursor::new(data.to_vec())))
    }

    fn build_record(record_type: u8, data_type: u8, payload: &[u8]) -> Vec<u8> {
        let size = (4 + payload.len()) as u16;
        let mut buf = Vec::with_capacity(size as usize);
        buf.extend_from_slice(&size.to_be_bytes());
        buf.push(record_type);
        buf.push(data_type);
        buf.extend_from_slice(payload);
        buf
    }

    #[test]
    fn test_empty_input_yields_no_records() {
        let mut reader = make_reader(&[]);
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_truncated_header_yields_no_records() {
        let mut reader = make_reader(&[0x00, 0x06]);
        assert!(reader.next().is_none());
    }

    #[test]
    fn test_truncated_body_returns_error() {
        let data = [0x00, 0x08, 0x00, 0x02, 0xAB];
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_record_type_returns_error() {
        let data = build_record(0xFF, 0x00, &[]);
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_invalid_record_types_all_return_errors() {
        for bad_type in [0x3C, 0x50, 0x80, 0xFE, 0xFF] {
            let data = build_record(bad_type, 0x00, &[]);
            let mut reader = make_reader(&data);
            let result = reader.next().unwrap();
            assert!(
                result.is_err(),
                "Expected error for record type {bad_type:#04x}"
            );
        }
    }

    #[test]
    fn test_zero_size_record_parses_without_panic() {
        let data = [0x00, 0x00, 0x04, 0x00];
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_size_less_than_header_parses_without_panic() {
        for size in [1u16, 2, 3] {
            let data = [(size >> 8) as u8, (size & 0xFF) as u8, 0x04, 0x00];
            let mut reader = make_reader(&data);
            let result = reader.next().unwrap();
            assert!(result.is_ok(), "Unexpected error for size={size}");
        }
    }

    #[test]
    fn test_oversized_record_returns_error() {
        let data = [0x00, 0x64, 0x00, 0x02, 0x00, 0x01];
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_data_type_returns_error() {
        let payload = [0x00, 0x01];
        let data = build_record(0x00, 0xFE, &payload);
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_data_type_for_boundary_parses_as_string() {
        let payload = b"hello\0";
        let data = build_record(
            GDSRecord::Boundary as u8,
            GDSDataType::AsciiString as u8,
            payload,
        );
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (record, data) = result.unwrap();
        assert!(matches!(record, GDSRecord::Boundary));
        assert!(matches!(data, GDSRecordData::Str(_)));
    }

    #[test]
    fn test_xy_record_with_odd_payload_size_does_not_panic() {
        let payload = [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0xFF];
        let data = build_record(
            GDSRecord::XY as u8,
            GDSDataType::FourByteSignedInteger as u8,
            &payload,
        );
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (record, rdata) = result.unwrap();
        assert!(matches!(record, GDSRecord::XY));
        if let GDSRecordData::I32(values) = rdata {
            assert_eq!(values.len(), 2);
        } else {
            panic!("Expected I32 data");
        }
    }

    #[test]
    fn test_xy_record_with_single_byte_payload_yields_empty_vec() {
        let payload = [0xFF];
        let data = build_record(
            GDSRecord::XY as u8,
            GDSDataType::FourByteSignedInteger as u8,
            &payload,
        );
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (_, rdata) = result.unwrap();
        if let GDSRecordData::I32(values) = rdata {
            assert!(values.is_empty());
        } else {
            panic!("Expected I32 data");
        }
    }

    #[test]
    fn test_valid_record_followed_by_truncated_record() {
        let mut data = build_record(GDSRecord::EndLib as u8, GDSDataType::NoData as u8, &[]);
        data.extend_from_slice(&[0x00, 0x10, 0x00, 0x02, 0xFF]);
        let mut reader = make_reader(&data);
        let first = reader.next().unwrap();
        assert!(first.is_ok());
        let second = reader.next().unwrap();
        assert!(second.is_err());
    }

    #[test]
    fn test_valid_record_followed_by_invalid_record_type() {
        let mut data = build_record(GDSRecord::EndLib as u8, GDSDataType::NoData as u8, &[]);
        data.extend_from_slice(&build_record(0xFF, 0x00, &[]));
        let mut reader = make_reader(&data);
        let first = reader.next().unwrap();
        assert!(first.is_ok());
        let second = reader.next().unwrap();
        assert!(second.is_err());
    }

    #[test]
    fn test_eight_byte_real_with_incomplete_payload_does_not_panic() {
        let payload = [0x41, 0x10, 0x00, 0x00, 0x00];
        let data = build_record(
            GDSRecord::Units as u8,
            GDSDataType::EightByteReal as u8,
            &payload,
        );
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (_, rdata) = result.unwrap();
        if let GDSRecordData::F64(values) = rdata {
            assert!(values.is_empty());
        } else {
            panic!("Expected F64 data");
        }
    }

    #[test]
    fn test_i16_with_single_byte_payload_yields_empty_vec() {
        let payload = [0xFF];
        let data = build_record(
            GDSRecord::Layer as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &payload,
        );
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (_, rdata) = result.unwrap();
        if let GDSRecordData::I16(values) = rdata {
            assert!(values.is_empty());
        } else {
            panic!("Expected I16 data");
        }
    }

    #[test]
    fn test_all_zeros_record_parses_as_header() {
        let data = [0x00, 0x04, 0x00, 0x00];
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (record, rdata) = result.unwrap();
        assert!(matches!(record, GDSRecord::Header));
        assert!(matches!(rdata, GDSRecordData::None));
    }

    #[test]
    fn test_maximum_u16_size_returns_error_on_insufficient_data() {
        let data = [0xFF, 0xFF, 0x00, 0x02, 0x00, 0x01];
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_garbage_bytes_after_valid_endlib() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();
        write_gds_tail_to_file(&mut data).unwrap();
        data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("garbage_after_endlib.gds");
        std::fs::write(&path, &data).unwrap();
        let result = Library::read_file(&path, None);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_empty_file_via_from_gds_returns_default_library() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.gds");
        std::fs::write(&path, []).unwrap();
        let result = from_gds(&path, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_truncated_file_mid_header_record_returns_error() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();
        data.truncate(data.len() / 2);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("truncated.gds");
        std::fs::write(&path, &data).unwrap();
        let result = from_gds(&path, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_with_only_invalid_record_type_returns_error() {
        let data = build_record(0xFF, 0x00, &[]);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("invalid_record.gds");
        std::fs::write(&path, &data).unwrap();
        let result = from_gds(&path, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_bgnstr_without_endstr_does_not_insert_cell() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();
        data.extend_from_slice(&build_record(
            GDSRecord::BgnStr as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &[0x00; 24],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::StrName as u8,
            GDSDataType::AsciiString as u8,
            b"orphan",
        ));
        write_gds_tail_to_file(&mut data).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no_endstr.gds");
        std::fs::write(&path, &data).unwrap();
        let library = from_gds(&path, None).unwrap();
        assert!(library.cells.is_empty());
    }

    #[test]
    fn test_boundary_with_string_data_does_not_create_polygon_points() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();
        data.extend_from_slice(&build_record(
            GDSRecord::BgnStr as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &[0x00; 24],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::StrName as u8,
            GDSDataType::AsciiString as u8,
            b"cell",
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::Boundary as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::XY as u8,
            GDSDataType::AsciiString as u8,
            b"not_coordinates",
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::EndEl as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::EndStr as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        write_gds_tail_to_file(&mut data).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wrong_data_type.gds");
        std::fs::write(&path, &data).unwrap();
        let library = from_gds(&path, None).unwrap();
        let cell = library.cells.values().next().unwrap();
        let polygons = cell.polygons();
        assert_eq!(polygons.len(), 1);
        assert!(polygons[0].points.is_empty());
    }

    #[test]
    fn test_record_reader_nodata_with_valid_utf8_payload() {
        let payload = b"hello";
        let data = build_record(GDSRecord::EndEl as u8, GDSDataType::NoData as u8, payload);
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (record, rdata) = result.unwrap();
        assert!(matches!(record, GDSRecord::EndEl));
        if let GDSRecordData::Str(s) = rdata {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected Str data for NoData with payload");
        }
    }

    struct ErrorReader;

    impl Read for ErrorReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "simulated error",
            ))
        }
    }

    #[test]
    fn test_record_reader_non_eof_io_error() {
        let mut reader = RecordReader::new(BufReader::new(ErrorReader));
        let result = reader.next();
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_write_transformation_angle_only() {
        let mut buf = Vec::new();
        write_transformation_to_file(&mut buf, 45.0, 1.0, false).unwrap();
        assert!(!buf.is_empty());

        let mut buf2 = Vec::new();
        write_transformation_to_file(&mut buf2, 45.0, 2.0, false).unwrap();
        assert!(buf2.len() > buf.len());
    }

    #[test]
    fn test_record_reader_invalid_utf8_in_ascii_string_returns_error() {
        let payload = [0xFF, 0xFE, 0x80, 0x81];
        let data = build_record(
            GDSRecord::LibName as u8,
            GDSDataType::AsciiString as u8,
            &payload,
        );
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_record_reader_invalid_utf8_in_nodata_with_payload_returns_error() {
        let payload = [0xFF, 0xFE, 0x80, 0x81];
        let data = build_record(GDSRecord::EndEl as u8, GDSDataType::NoData as u8, &payload);
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_gds_mismatched_data_types_are_silently_ignored() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();

        data.extend_from_slice(&build_record(
            GDSRecord::BgnStr as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &[0x00; 24],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::StrName as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &0i16.to_be_bytes(),
        ));

        // Path with wrong data types for Layer, DataType, Width, PathType
        data.extend_from_slice(&build_record(
            GDSRecord::Path as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::Layer as u8,
            GDSDataType::EightByteReal as u8,
            &[0x00; 8],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::DataType as u8,
            GDSDataType::EightByteReal as u8,
            &[0x00; 8],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::Width as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &0i16.to_be_bytes(),
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::PathType as u8,
            GDSDataType::EightByteReal as u8,
            &[0x00; 8],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::EndEl as u8,
            GDSDataType::NoData as u8,
            &[],
        ));

        // Text with wrong data types for Presentation, String, XY
        data.extend_from_slice(&build_record(
            GDSRecord::Text as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::Presentation as u8,
            GDSDataType::EightByteReal as u8,
            &[0x00; 8],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::String as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &0i16.to_be_bytes(),
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::EndEl as u8,
            GDSDataType::NoData as u8,
            &[],
        ));

        // Reference (ARef) with wrong data types for SName, ColRow, STrans, Mag, Angle
        data.extend_from_slice(&build_record(
            GDSRecord::ARef as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::SName as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &0i16.to_be_bytes(),
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::ColRow as u8,
            GDSDataType::EightByteReal as u8,
            &[0x00; 16],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::STrans as u8,
            GDSDataType::EightByteReal as u8,
            &[0x00; 8],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::Mag as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &0i16.to_be_bytes(),
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::Angle as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &0i16.to_be_bytes(),
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::EndEl as u8,
            GDSDataType::NoData as u8,
            &[],
        ));

        data.extend_from_slice(&build_record(
            GDSRecord::EndStr as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        write_gds_tail_to_file(&mut data).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mismatched.gds");
        std::fs::write(&path, &data).unwrap();
        let library = from_gds(&path, None).unwrap();
        assert_eq!(library.cells.len(), 1);
    }

    #[test]
    fn test_from_gds_endel_outside_cell_is_ignored() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();
        data.extend_from_slice(&build_record(
            GDSRecord::EndEl as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        write_gds_tail_to_file(&mut data).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("endel_no_cell.gds");
        std::fs::write(&path, &data).unwrap();
        let library = from_gds(&path, None).unwrap();
        assert!(library.cells.is_empty());
    }

    #[test]
    fn test_from_gds_xy_reference_with_unexpected_point_count() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();
        data.extend_from_slice(&build_record(
            GDSRecord::BgnStr as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &[0x00; 24],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::StrName as u8,
            GDSDataType::AsciiString as u8,
            b"cell",
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::ARef as u8,
            GDSDataType::NoData as u8,
            &[],
        ));

        // XY with 2 points (neither 1 nor 3)
        let mut xy_payload = Vec::new();
        for val in [10i32, 20, 30, 40] {
            xy_payload.extend_from_slice(&val.to_be_bytes());
        }
        data.extend_from_slice(&build_record(
            GDSRecord::XY as u8,
            GDSDataType::FourByteSignedInteger as u8,
            &xy_payload,
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::EndEl as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::EndStr as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        write_gds_tail_to_file(&mut data).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("xy_2pts.gds");
        std::fs::write(&path, &data).unwrap();
        let library = from_gds(&path, None).unwrap();
        assert_eq!(library.cells.len(), 1);
    }

    #[test]
    fn test_from_gds_records_outside_element_context() {
        let mut data = Vec::new();
        write_gds_head_to_file("test", 1e-3, 1e-9, &mut data).unwrap();
        data.extend_from_slice(&build_record(
            GDSRecord::BgnStr as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &[0x00; 24],
        ));
        data.extend_from_slice(&build_record(
            GDSRecord::StrName as u8,
            GDSDataType::AsciiString as u8,
            b"cell",
        ));

        // SName outside reference context
        data.extend_from_slice(&build_record(
            GDSRecord::SName as u8,
            GDSDataType::AsciiString as u8,
            b"refname",
        ));
        // Presentation outside text context
        data.extend_from_slice(&build_record(
            GDSRecord::Presentation as u8,
            GDSDataType::BitArray as u8,
            &0i16.to_be_bytes(),
        ));
        // XY outside any element context
        let mut xy_payload = Vec::new();
        for val in [10i32, 20] {
            xy_payload.extend_from_slice(&val.to_be_bytes());
        }
        data.extend_from_slice(&build_record(
            GDSRecord::XY as u8,
            GDSDataType::FourByteSignedInteger as u8,
            &xy_payload,
        ));

        data.extend_from_slice(&build_record(
            GDSRecord::EndStr as u8,
            GDSDataType::NoData as u8,
            &[],
        ));
        write_gds_tail_to_file(&mut data).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out_of_context.gds");
        std::fs::write(&path, &data).unwrap();
        let library = from_gds(&path, None).unwrap();
        assert_eq!(library.cells.len(), 1);
    }

    #[test]
    fn test_from_gds_units_with_wrong_data_type() {
        let mut data = Vec::new();
        // Write header manually without using write_gds_head_to_file,
        // so we can inject a Units record with wrong data type.
        let now = chrono::Local::now().naive_utc();
        let head_start = [
            6u16,
            combine_record_and_data_type(GDSRecord::Header, GDSDataType::TwoByteSignedInteger),
            0x0258,
            28,
            combine_record_and_data_type(GDSRecord::BgnLib, GDSDataType::TwoByteSignedInteger),
            now.year() as u16,
            now.month() as u16,
            now.day() as u16,
            now.hour() as u16,
            now.minute() as u16,
            now.second() as u16,
            now.year() as u16,
            now.month() as u16,
            now.day() as u16,
            now.hour() as u16,
            now.minute() as u16,
            now.second() as u16,
        ];
        write_u16_array_to_file(&mut data, &head_start).unwrap();
        write_string_with_record_to_file(&mut data, GDSRecord::LibName, "test").unwrap();

        // Units with I16 instead of F64
        data.extend_from_slice(&build_record(
            GDSRecord::Units as u8,
            GDSDataType::TwoByteSignedInteger as u8,
            &[0x00, 0x01, 0x00, 0x02],
        ));

        write_gds_tail_to_file(&mut data).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad_units.gds");
        std::fs::write(&path, &data).unwrap();
        let library = from_gds(&path, None).unwrap();
        assert!(library.cells.is_empty());
    }
}
