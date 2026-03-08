use std::fs::File;
use std::io::{self, BufReader, Read};

use crate::cell::Cell;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, GDSRecordData, STRANS_X_REFLECTION};
use crate::elements::text::get_presentations_from_value;
use crate::elements::{GdsBox, Node, Path, PathType, Polygon, Reference, Text};
use crate::error::GdsError;
use crate::geometry::round_to_decimals;
use crate::library::Library;
use crate::{DEFAULT_INTEGER_UNITS, DataType, Instance, Layer, Point, Unit};

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
    let mut gds_box: Option<GdsBox> = None;
    let mut node: Option<Node> = None;
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
                GDSRecord::Boundary => {
                    polygon = Some(Polygon::default());
                }
                GDSRecord::Box => {
                    gds_box = Some(GdsBox::default());
                }
                GDSRecord::Node => {
                    node = Some(Node::default());
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
                        let layer_value = Layer::new(layer[0] as u16);
                        if let Some(polygon) = &mut polygon {
                            polygon.layer = layer_value;
                        } else if let Some(gds_box) = &mut gds_box {
                            gds_box.layer = layer_value;
                        } else if let Some(node) = &mut node {
                            node.layer = layer_value;
                        } else if let Some(path) = &mut path {
                            path.layer = layer_value;
                        } else if let Some(text) = &mut text {
                            text.layer = layer_value;
                        }
                    }
                }
                GDSRecord::DataType => {
                    if let GDSRecordData::I16(data_type) = data {
                        let data_type_val = DataType::new(data_type[0] as u16);
                        if let Some(polygon) = &mut polygon {
                            polygon.data_type = data_type_val;
                        } else if let Some(path) = &mut path {
                            path.data_type = data_type_val;
                        }
                    }
                }
                GDSRecord::BoxType => {
                    if let GDSRecordData::I16(data_type) = data {
                        if let Some(gds_box) = &mut gds_box {
                            gds_box.box_type = DataType::new(data_type[0] as u16);
                        }
                    }
                }
                GDSRecord::NodeType => {
                    if let GDSRecordData::I16(data_type) = data {
                        if let Some(node) = &mut node {
                            node.node_type = DataType::new(data_type[0] as u16);
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
                        } else if let Some(gds_box) = &mut gds_box {
                            let (min, max) = crate::geometry::bounding_box(&points);
                            gds_box.bottom_left = min;
                            gds_box.top_right = max;
                        } else if let Some(node) = &mut node {
                            node.points = points;
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
                                                (unrotated_points[1] - unrotated_points[0])
                                                    / reference.grid.columns(),
                                            )
                                        } else {
                                            None
                                        });
                                    reference.grid.set_spacing_y(if reference.grid.rows() > 1 {
                                        Some(
                                            (unrotated_points[2] - unrotated_points[0])
                                                / reference.grid.rows(),
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
                        } else if let Some(gds_box) = gds_box.take() {
                            cell.add(gds_box);
                        } else if let Some(node) = node.take() {
                            cell.add(node);
                        } else if let Some(path) = path.take() {
                            cell.add(path);
                        } else if let Some(reference) = reference.take() {
                            cell.add(reference);
                        } else if let Some(text) = text.take() {
                            cell.add(text);
                        }
                    }
                    polygon = None;
                    gds_box = None;
                    node = None;
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
                        let x_reflection = flags[0] & STRANS_X_REFLECTION as i16 != 0;
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
                            reference.grid.set_angle(angle[0].to_radians());
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
                GDSRecord::BgnExtn => {
                    if let GDSRecordData::I32(extn) = data {
                        if let Some(path) = &mut path {
                            let value = round_to_decimals(f64::from(extn[0]) * scale, 10);
                            path.begin_extension = Some(Unit::float(value, db_units));
                        }
                    }
                }
                GDSRecord::EndExtn => {
                    if let GDSRecordData::I32(extn) = data {
                        if let Some(path) = &mut path {
                            let value = round_to_decimals(f64::from(extn[0]) * scale, 10);
                            path.end_extension = Some(Unit::float(value, db_units));
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
