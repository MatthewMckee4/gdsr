use std::io::{self, BufReader, Read};

use crate::cell::Cell;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, GDSRecordData, STRANS_X_REFLECTION};
use crate::elements::text::get_presentations_from_value;
use crate::elements::{GdsBox, Node, Path, PathType, Polygon, Property, Reference, Text};
use crate::error::GdsError;
use crate::geometry::round_to_decimals;
use crate::library::Library;
use crate::{
    DEFAULT_INTEGER_UNITS, DataType, Degrees, GdsTimestamps, Instance, Layer, Point, Unit,
};

pub fn from_gds_reader<R: Read>(reader: R, units: Option<f64>) -> Result<Library, GdsError> {
    from_gds_reader_filtered(reader, units, |_, _| true)
}

#[allow(clippy::too_many_lines)]
pub fn from_gds_reader_filtered<R, F>(
    reader: R,
    units: Option<f64>,
    layer_filter: F,
) -> Result<Library, GdsError>
where
    R: Read,
    F: Fn(Layer, DataType) -> bool,
{
    let mut library = Library::new("Library");

    let reader = RecordReader::new(BufReader::new(reader));

    let mut cell: Option<Cell> = None;
    let mut path: Option<Path> = None;
    let mut polygon: Option<Polygon> = None;
    let mut gds_box: Option<GdsBox> = None;
    let mut node: Option<Node> = None;
    let mut text: Option<Text> = None;
    let mut reference: Option<Reference> = None;
    let mut property_attribute: Option<u16> = None;
    let mut element_open = false;
    let mut end_library_seen = false;

    let mut scale = 1.0;
    let mut db_units = units.unwrap_or(DEFAULT_INTEGER_UNITS);

    for record in reader {
        match record {
            Ok((record_type, data)) => match record_type {
                GDSRecord::BgnLib => {
                    let GDSRecordData::I16(values) = data else {
                        return Err(invalid_timestamp_record("BGNLIB"));
                    };
                    library.timestamps = Some(GdsTimestamps::from_record(&values, "BGNLIB")?);
                }
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
                    let GDSRecordData::I16(values) = data else {
                        return Err(invalid_timestamp_record("BGNSTR"));
                    };
                    let mut new_cell = Cell::default();
                    new_cell.set_timestamps(GdsTimestamps::from_record(&values, "BGNSTR")?);
                    cell = Some(new_cell);
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
                GDSRecord::EndLib => end_library_seen = true,
                GDSRecord::Boundary => {
                    property_attribute = None;
                    polygon = Some(Polygon::default());
                    element_open = true;
                }
                GDSRecord::Box => {
                    property_attribute = None;
                    gds_box = Some(GdsBox::default());
                    element_open = true;
                }
                GDSRecord::Node => {
                    property_attribute = None;
                    node = Some(Node::default());
                    element_open = true;
                }
                GDSRecord::Path => {
                    property_attribute = None;
                    path = Some(Path::default());
                    element_open = true;
                }
                GDSRecord::ARef | GDSRecord::SRef => {
                    property_attribute = None;
                    reference = Some(Reference::default());
                    element_open = true;
                }
                GDSRecord::Text => {
                    property_attribute = None;
                    text = Some(Text::default());
                    element_open = true;
                }
                GDSRecord::TextNode => element_open = true,
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
                GDSRecord::TextType => {
                    if let GDSRecordData::I16(data_type) = data {
                        if let Some(text) = &mut text {
                            text.datatype = DataType::new(data_type[0] as u16);
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
                        if !should_parse_xy(
                            &layer_filter,
                            polygon.as_ref(),
                            gds_box.as_ref(),
                            node.as_ref(),
                            path.as_ref(),
                            text.as_ref(),
                            reference.as_ref(),
                        ) {
                            continue;
                        }

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
                            if layer_filter(polygon.layer, polygon.data_type) {
                                cell.add(polygon);
                            }
                        } else if let Some(gds_box) = gds_box.take() {
                            if layer_filter(gds_box.layer, gds_box.box_type) {
                                cell.add(gds_box);
                            }
                        } else if let Some(node) = node.take() {
                            if layer_filter(node.layer, node.node_type) {
                                cell.add(node);
                            }
                        } else if let Some(path) = path.take() {
                            if layer_filter(path.layer, path.data_type) {
                                cell.add(path);
                            }
                        } else if let Some(reference) = reference.take() {
                            cell.add(reference);
                        } else if let Some(text) = text.take() {
                            if layer_filter(text.layer, text.datatype) {
                                cell.add(text);
                            }
                        }
                    }
                    polygon = None;
                    gds_box = None;
                    node = None;
                    path = None;
                    text = None;
                    reference = None;
                    property_attribute = None;
                    element_open = false;
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
                        let radians = Degrees::new(angle[0]).to_radians();
                        if let Some(text) = &mut text {
                            text.angle = radians;
                        } else if let Some(reference) = &mut reference {
                            reference.grid.set_angle(radians);
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
                GDSRecord::PropAttr => {
                    if let GDSRecordData::I16(attributes) = data
                        && let Some(attribute) = attributes.first()
                    {
                        property_attribute = Some(*attribute as u16);
                    }
                }
                GDSRecord::PropValue => {
                    if let GDSRecordData::Str(value) = data
                        && let Some(attribute) = property_attribute
                    {
                        let property = Property::new(attribute, value);
                        if let Some(polygon) = &mut polygon {
                            polygon.properties_mut().push(property);
                        } else if let Some(gds_box) = &mut gds_box {
                            gds_box.properties_mut().push(property);
                        } else if let Some(node) = &mut node {
                            node.properties_mut().push(property);
                        } else if let Some(path) = &mut path {
                            path.properties_mut().push(property);
                        } else if let Some(reference) = &mut reference {
                            reference.properties_mut().push(property);
                        } else if let Some(text) = &mut text {
                            text.properties_mut().push(property);
                        }
                    }
                }
                _ => {}
            },
            Err(e) => return Err(e),
        }
    }

    if element_open {
        return Err(invalid_data("Unexpected EOF before ENDEL record"));
    }
    if cell.is_some() {
        return Err(invalid_data("Unexpected EOF before ENDSTR record"));
    }
    if !end_library_seen {
        return Err(invalid_data("Unexpected EOF before ENDLIB record"));
    }

    Ok(library)
}

fn invalid_data(message: impl Into<String>) -> GdsError {
    GdsError::InvalidData {
        message: message.into(),
    }
}

fn invalid_timestamp_record(record: &str) -> GdsError {
    invalid_data(format!("Invalid {record} timestamp record"))
}

fn should_parse_xy<F>(
    layer_filter: &F,
    polygon: Option<&Polygon>,
    gds_box: Option<&GdsBox>,
    node: Option<&Node>,
    path: Option<&Path>,
    text: Option<&Text>,
    reference: Option<&Reference>,
) -> bool
where
    F: Fn(Layer, DataType) -> bool,
{
    if reference.is_some() {
        return true;
    }
    if let Some(polygon) = polygon {
        return layer_filter(polygon.layer, polygon.data_type);
    }
    if let Some(gds_box) = gds_box {
        return layer_filter(gds_box.layer, gds_box.box_type);
    }
    if let Some(node) = node {
        return layer_filter(node.layer, node.node_type);
    }
    if let Some(path) = path {
        return layer_filter(path.layer, path.data_type);
    }
    if let Some(text) = text {
        return layer_filter(text.layer, text.datatype);
    }
    true
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
        if let Err(e) = self.reader.read_exact(&mut header[..1]) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return None;
            }
            return Some(Err(GdsError::from(e)));
        }
        if let Err(error) = self.reader.read_exact(&mut header[1..]) {
            return Some(Err(if error.kind() == io::ErrorKind::UnexpectedEof {
                invalid_data("Truncated record header")
            } else {
                GdsError::from(error)
            }));
        }

        let size = u16::from_be_bytes([header[0], header[1]]) as usize;
        if size < 4 {
            return Some(Err(invalid_data(format!(
                "Record size {size} is smaller than the four-byte header"
            ))));
        }

        let Ok(record) = GDSRecord::try_from(header[2]) else {
            return Some(Err(invalid_data(format!(
                "Invalid record type byte: {:#04x}",
                header[2]
            ))));
        };
        let Ok(data_type) = GDSDataType::try_from(header[3]) else {
            return Some(Err(invalid_data(format!(
                "Invalid data type byte: {:#04x}",
                header[3]
            ))));
        };

        let mut buf = vec![0u8; size - 4];
        if let Err(error) = self.reader.read_exact(&mut buf) {
            return Some(Err(if error.kind() == io::ErrorKind::UnexpectedEof {
                invalid_data(format!("Truncated {record:?} record payload"))
            } else {
                GdsError::from(error)
            }));
        }
        if let Err(error) = validate_record_layout(record, data_type, buf.len()) {
            return Some(Err(error));
        }

        let data = match data_type {
            GDSDataType::TwoByteSignedInteger | GDSDataType::BitArray => {
                GDSRecordData::I16(read_i16_be(&buf))
            }
            GDSDataType::FourByteSignedInteger | GDSDataType::FourByteReal => {
                GDSRecordData::I32(read_i32_be(&buf))
            }
            GDSDataType::EightByteReal => GDSRecordData::F64(
                read_u64_be(&buf)
                    .into_iter()
                    .map(eight_byte_real_to_float)
                    .collect(),
            ),
            GDSDataType::AsciiString => match String::from_utf8(buf) {
                Ok(mut result) => {
                    if result.ends_with('\0') {
                        result.pop();
                    }
                    GDSRecordData::Str(result)
                }
                Err(error) => {
                    return Some(Err(invalid_data(format!(
                        "Invalid UTF-8 in ASCII string record: {error}"
                    ))));
                }
            },
            GDSDataType::NoData => GDSRecordData::None,
        };

        Some(Ok((record, data)))
    }
}

fn validate_record_layout(
    record: GDSRecord,
    data_type: GDSDataType,
    payload_len: usize,
) -> Result<(), GdsError> {
    let value_size = usize::from(data_type.byte_size());
    if value_size > 0 && !payload_len.is_multiple_of(value_size) {
        return Err(invalid_data(format!(
            "{record:?} payload length {payload_len} is not aligned to {value_size} bytes"
        )));
    }
    if data_type == GDSDataType::NoData && payload_len != 0 {
        return Err(invalid_data(format!(
            "{record:?} NoData record has a non-empty payload"
        )));
    }

    let expected = match record {
        GDSRecord::Header => Some((GDSDataType::TwoByteSignedInteger, 1)),
        GDSRecord::BgnLib | GDSRecord::BgnStr => Some((GDSDataType::TwoByteSignedInteger, 12)),
        GDSRecord::Units => Some((GDSDataType::EightByteReal, 2)),
        GDSRecord::Layer
        | GDSRecord::DataType
        | GDSRecord::TextType
        | GDSRecord::NodeType
        | GDSRecord::PathType
        | GDSRecord::PropAttr
        | GDSRecord::BoxType => Some((GDSDataType::TwoByteSignedInteger, 1)),
        GDSRecord::Presentation | GDSRecord::STrans => Some((GDSDataType::BitArray, 1)),
        GDSRecord::ColRow => Some((GDSDataType::TwoByteSignedInteger, 2)),
        GDSRecord::Width | GDSRecord::BgnExtn | GDSRecord::EndExtn => {
            Some((GDSDataType::FourByteSignedInteger, 1))
        }
        GDSRecord::Mag | GDSRecord::Angle => Some((GDSDataType::EightByteReal, 1)),
        GDSRecord::EndLib
        | GDSRecord::EndStr
        | GDSRecord::Boundary
        | GDSRecord::Path
        | GDSRecord::SRef
        | GDSRecord::ARef
        | GDSRecord::Text
        | GDSRecord::EndEl
        | GDSRecord::TextNode
        | GDSRecord::Node
        | GDSRecord::Box
        | GDSRecord::EndMasks => Some((GDSDataType::NoData, 0)),
        _ => None,
    };

    if let Some((expected_type, expected_count)) = expected {
        let actual_count = payload_len.checked_div(value_size).unwrap_or(0);
        if data_type != expected_type || actual_count != expected_count {
            return Err(invalid_data(format!(
                "Invalid {record:?} payload: expected {expected_count} {expected_type:?} value(s)"
            )));
        }
    }
    if record == GDSRecord::XY
        && (data_type != GDSDataType::FourByteSignedInteger
            || !(payload_len / usize::from(GDSDataType::FourByteSignedInteger.byte_size()))
                .is_multiple_of(2))
    {
        return Err(invalid_data(
            "Invalid XY payload: expected coordinate pairs of four-byte integers",
        ));
    }

    Ok(())
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
    use std::io::{BufReader, Cursor, Write};

    use quickcheck_macros::quickcheck;

    use super::*;
    use crate::GdsTimestampPolicy;

    fn record(record: GDSRecord, data_type: GDSDataType, payload: &[u8]) -> Vec<u8> {
        let size = 4 + payload.len() as u16;
        let mut bytes = Vec::with_capacity(usize::from(size));
        bytes.extend_from_slice(&size.to_be_bytes());
        bytes.push(record as u8);
        bytes.push(data_type as u8);
        bytes.extend_from_slice(payload);
        bytes
    }

    fn assert_invalid(result: &Result<Library, GdsError>) {
        assert!(matches!(result, Err(GdsError::InvalidData { .. })));
    }

    #[test]
    fn malformed_record_layouts_return_invalid_data() {
        let cases = [
            ("one-byte header", vec![0]),
            ("two-byte header", vec![0, 4]),
            ("three-byte header", vec![0, 4, GDSRecord::EndLib as u8]),
            (
                "undersized record",
                vec![0, 3, GDSRecord::EndLib as u8, GDSDataType::NoData as u8],
            ),
            (
                "truncated payload",
                vec![
                    0,
                    6,
                    GDSRecord::Layer as u8,
                    GDSDataType::TwoByteSignedInteger as u8,
                    0,
                ],
            ),
            (
                "misaligned integer payload",
                vec![
                    0,
                    7,
                    GDSRecord::Layer as u8,
                    GDSDataType::TwoByteSignedInteger as u8,
                    0,
                    0,
                    0,
                ],
            ),
            (
                "empty scalar payload",
                record(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger, &[]),
            ),
            (
                "short fixed-cardinality payload",
                record(
                    GDSRecord::ColRow,
                    GDSDataType::TwoByteSignedInteger,
                    &[0, 1],
                ),
            ),
            (
                "wrong scalar data type",
                record(
                    GDSRecord::Layer,
                    GDSDataType::FourByteSignedInteger,
                    &[0, 0, 0, 1],
                ),
            ),
            (
                "odd XY coordinate count",
                record(
                    GDSRecord::XY,
                    GDSDataType::FourByteSignedInteger,
                    &[0, 0, 0, 1],
                ),
            ),
        ];

        for (name, bytes) in cases {
            let mut reader = RecordReader::new(BufReader::new(Cursor::new(bytes)));
            assert!(
                matches!(reader.next(), Some(Err(GdsError::InvalidData { .. }))),
                "{name}"
            );
        }
    }

    #[test]
    fn every_proper_prefix_of_minimal_library_is_rejected() {
        let bytes = Library::new("minimal")
            .to_bytes_with_timestamp_policy(1e-3, 1e-9, GdsTimestampPolicy::Zero)
            .expect("minimal library should serialize");

        for end in 0..bytes.len() {
            assert_invalid(&Library::from_bytes(&bytes[..end], None));
        }
        Library::from_bytes(&bytes, None).expect("complete minimal library should parse");
    }

    #[test]
    fn eof_before_each_structural_terminator_is_rejected() {
        let minimal = Library::new("minimal")
            .to_bytes_with_timestamp_policy(1e-3, 1e-9, GdsTimestampPolicy::Zero)
            .expect("minimal library should serialize");
        let missing_end_library = &minimal[..minimal.len() - 4];
        let missing_end_structure = record(
            GDSRecord::BgnStr,
            GDSDataType::TwoByteSignedInteger,
            &[0; 24],
        );
        let missing_end_element = record(GDSRecord::Boundary, GDSDataType::NoData, &[]);

        for (terminator, bytes) in [
            ("ENDLIB", missing_end_library),
            ("ENDSTR", missing_end_structure.as_slice()),
            ("ENDEL", missing_end_element.as_slice()),
        ] {
            let error = Library::from_bytes(bytes, None).expect_err("stream should be incomplete");
            assert!(
                matches!(
                    error,
                    GdsError::InvalidData { ref message } if message.contains(terminator)
                ),
                "{terminator}: {error}"
            );
        }
    }

    #[test]
    fn all_library_read_entry_points_reject_malformed_input() {
        let bytes = [0, 4];
        let mut file = tempfile::NamedTempFile::new().expect("temporary file should be created");
        file.write_all(&bytes)
            .expect("malformed test data should be written");
        file.flush().expect("malformed test data should be flushed");

        assert_invalid(&Library::from_bytes(&bytes, None));
        assert_invalid(&Library::read_from(Cursor::new(bytes), None));
        assert_invalid(&Library::read_from_filtered(
            Cursor::new(bytes),
            None,
            |_, _| true,
        ));
        assert_invalid(&Library::read_file(file.path(), None));
        assert_invalid(&Library::read_file_filtered(file.path(), None, |_, _| true));
    }

    #[quickcheck]
    fn generated_record_never_panics(record_type: u8, data_type: u8, mut payload: Vec<u8>) -> bool {
        payload.truncate(256);
        let size = 4 + payload.len() as u16;
        let mut bytes = Vec::with_capacity(usize::from(size) + 4);
        bytes.extend_from_slice(&size.to_be_bytes());
        bytes.push(record_type);
        bytes.push(data_type);
        bytes.extend_from_slice(&payload);
        bytes.extend_from_slice(&[0, 4, GDSRecord::EndLib as u8, GDSDataType::NoData as u8]);

        std::panic::catch_unwind(|| Library::from_bytes(&bytes, None)).is_ok()
    }
}
