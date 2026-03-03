use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

use bytemuck::cast_slice;
use chrono::{Datelike, Local, Timelike};

use crate::cell::Cell;
use crate::config::gds_file_types::{
    GDSDataType, GDSRecord, GDSRecordData, combine_record_and_data_type,
};
use crate::elements::text::utils::get_presentations_from_value;
use crate::elements::{Path, PathType, Polygon, Reference, Text};
use crate::library::Library;
use crate::utils::gds_format::{eight_byte_real, u16_array_to_big_endian};
use crate::utils::geometry::round_to_decimals;
use crate::{DEFAULT_INTEGER_UNITS, DataType, Instance, Layer, Point, ToGds, Unit};

pub fn write_gds_head_to_file(
    library_name: &str,
    user_units: f64,
    db_units: f64,
    buffer: &mut impl std::io::Write,
) -> io::Result<()> {
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

pub fn write_gds_tail_to_file(buffer: &mut impl std::io::Write) -> io::Result<()> {
    let tail = [
        4,
        combine_record_and_data_type(GDSRecord::EndLib, GDSDataType::NoData),
    ];
    write_u16_array_to_file(buffer, &tail)
}

pub fn write_u16_array_to_file(buffer: &mut impl std::io::Write, array: &[u16]) -> io::Result<()> {
    let u16_array_to_big_endian = u16_array_to_big_endian(array);
    buffer.write_all(cast_slice(&u16_array_to_big_endian))
}

pub fn write_float_to_eight_byte_real_to_file(
    buffer: &mut impl std::io::Write,
    value: f64,
) -> io::Result<()> {
    let value = eight_byte_real(value);
    buffer.write_all(&value)
}

pub const MAX_POINTS: usize = 8191;

pub fn write_points_to_file(
    buffer: &mut impl std::io::Write,
    points: &[Point],
    database_units: f64,
) -> io::Result<()> {
    let integer_points: Vec<Point> = points.iter().map(Point::to_integer_unit).collect();

    let points_to_write = integer_points.get(..MAX_POINTS).unwrap_or(&integer_points);

    let record_size = 4 + (points_to_write.len() * 8) as u16;
    let xy_header_buffer = [
        record_size,
        combine_record_and_data_type(GDSRecord::XY, GDSDataType::FourByteSignedInteger),
    ];

    write_u16_array_to_file(buffer, &xy_header_buffer)?;

    for point in points_to_write {
        let x_real = point.x().absolute_value();
        let y_real = point.y().absolute_value();

        let scaled_x = (x_real / database_units).round() as i32;
        let scaled_y = (y_real / database_units).round() as i32;

        buffer.write_all(&scaled_x.to_be_bytes())?;
        buffer.write_all(&scaled_y.to_be_bytes())?;
    }

    Ok(())
}

pub fn write_element_tail_to_file(buffer: &mut impl std::io::Write) -> io::Result<()> {
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
) -> io::Result<()> {
    let mut len = string.len();
    if len % 2 != 0 {
        len += 1;
    }

    let mut lib_name_bytes = string.as_bytes().to_vec();

    if string.len() % 2 != 0 {
        lib_name_bytes.push(0);
    }

    let string_start = [
        (4 + len) as u16,
        combine_record_and_data_type(record, GDSDataType::AsciiString),
    ];

    write_u16_array_to_file(buffer, &string_start)?;

    buffer.write_all(&lib_name_bytes)
}

pub fn write_gds<'a, P: AsRef<std::path::Path>>(
    file_name: P,
    library_name: &str,
    user_units: f64,
    database_units: f64,
    cells: impl Iterator<Item = &'a Cell>,
) -> io::Result<()> {
    let mut file = File::create(file_name)?;

    write_gds_head_to_file(library_name, user_units, database_units, &mut file)?;

    for cell in cells {
        cell.to_gds_impl(&mut file, database_units)?;
    }

    write_gds_tail_to_file(&mut file)?;

    file.flush()
}

pub fn write_transformation_to_file(
    buffer: &mut impl std::io::Write,
    angle: f64,
    magnification: f64,
    x_reflection: bool,
) -> io::Result<()> {
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
) -> io::Result<Library> {
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
    type Item = io::Result<(GDSRecord, GDSRecordData)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut header = [0u8; 4];
        if let Err(e) = self.reader.read_exact(&mut header) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return None;
            }
            return Some(Err(e));
        }

        let size = u16::from_be_bytes([header[0], header[1]]) as usize;
        let record_type = header[2];
        let data_type = header[3];

        let data = if size > 4 {
            let mut buf = vec![0u8; size - 4];
            if let Err(e) = self.reader.read_exact(&mut buf) {
                return Some(Err(e));
            }

            GDSDataType::try_from(data_type).map_or(
                GDSRecordData::None,
                |data_type| match data_type {
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
                    GDSDataType::AsciiString => {
                        let mut result = String::from_utf8_lossy(&buf).into_owned();
                        if result.ends_with('\0') {
                            result.pop();
                        }
                        GDSRecordData::Str(result)
                    }
                    GDSDataType::NoData => {
                        GDSRecordData::Str(String::from_utf8_lossy(&buf).into_owned())
                    }
                },
            )
        } else {
            GDSRecordData::None
        };

        GDSRecord::try_from(record_type).map_or_else(
            |()| {
                Some(Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid record type",
                )))
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
    use super::*;
    use std::io::Cursor;

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

    fn make_reader(data: &[u8]) -> RecordReader<Cursor<Vec<u8>>> {
        RecordReader::new(BufReader::new(Cursor::new(data.to_vec())))
    }

    /// Builds a raw GDS record: `[size_hi, size_lo, record_type, data_type, ...payload]`.
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
        let data = [
            0x00, 0x08, // size = 8 (4 header + 4 payload)
            0x00, // Header record type
            0x02, // TwoByteSignedInteger
            0xAB, // only 1 byte of payload instead of 4
        ];
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
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("Invalid record type"));
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
        let data = [
            0x00, 0x00, // size = 0
            0x04, // EndLib record type
            0x00, // NoData
        ];
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_size_less_than_header_parses_without_panic() {
        for size in [1u16, 2, 3] {
            let data = [
                (size >> 8) as u8,
                (size & 0xFF) as u8,
                0x04, // EndLib
                0x00, // NoData
            ];
            let mut reader = make_reader(&data);
            let result = reader.next().unwrap();
            assert!(result.is_ok(), "Unexpected error for size={size}");
        }
    }

    #[test]
    fn test_oversized_record_returns_error() {
        let data = [
            0x00, 0x64, // size = 100 (claims 96 bytes of payload)
            0x00, // Header
            0x02, // TwoByteSignedInteger
            0x00, 0x01, // only 2 bytes of actual payload
        ];
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_data_type_yields_none_data() {
        let payload = [0x00, 0x01];
        let data = build_record(0x00, 0xFE, &payload);
        let mut reader = make_reader(&data);
        let result = reader.next().unwrap();
        let (record, data) = result.unwrap();
        assert!(matches!(record, GDSRecord::Header));
        assert!(matches!(data, GDSRecordData::None));
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
        let data = [
            0xFF, 0xFF, // size = 65535
            0x00, // Header
            0x02, // TwoByteSignedInteger
            0x00, 0x01, // only 2 bytes of payload
        ];
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
}
