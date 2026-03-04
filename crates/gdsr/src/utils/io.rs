use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

use chrono::{Datelike, Local, Timelike};

use crate::cell::Cell;
use crate::config::gds_file_types::{
    GDSDataType, GDSRecord, GDSRecordData, combine_record_and_data_type,
};
use crate::elements::text::get_presentations_from_value;
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
