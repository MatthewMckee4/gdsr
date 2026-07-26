use std::io::{self, BufRead, BufReader, Read};

use crate::cell::Cell;
use crate::config::gds_file_types::GDSDataType::{
    AsciiString, BitArray, EightByteReal, FourByteSignedInteger, NoData, TwoByteSignedInteger,
};
use crate::config::gds_file_types::{GDSDataType, GDSRecord, GDSRecordData, STRANS_X_REFLECTION};
use crate::elements::text::get_presentations_from_value;
use crate::elements::{GdsBox, Node, Path, PathType, Polygon, Property, Reference, Text};
use crate::error::GdsError;
use crate::geometry::round_to_decimals;
use crate::library::Library;
use crate::{
    DEFAULT_INTEGER_UNITS, DataType, Degrees, GdsTimestamps, Instance, Layer, Point, Unit,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParserState {
    Library,
    Structure,
    Element,
}

impl ParserState {
    #[inline]
    fn require(self, expected: Self, record: GDSRecord) -> Result<(), GdsError> {
        if self == expected {
            Ok(())
        } else {
            Err(invalid_parser_state(record, self))
        }
    }
}

#[cold]
fn invalid_parser_state(record: GDSRecord, state: ParserState) -> GdsError {
    invalid_data(format!(
        "Unexpected {record:?} record while parsing {state:?}"
    ))
}

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
    let mut state = ParserState::Library;

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
                    state.require(ParserState::Library, record_type)?;
                    let GDSRecordData::I16(values) = data else {
                        return Err(invalid_timestamp_record("BGNSTR"));
                    };
                    let mut new_cell = Cell::default();
                    new_cell.set_timestamps(GdsTimestamps::from_record(&values, "BGNSTR")?);
                    cell = Some(new_cell);
                    state = ParserState::Structure;
                }
                GDSRecord::StrName => {
                    if let GDSRecordData::Str(cell_name) = data {
                        if let Some(cell) = &mut cell {
                            cell.set_name(&cell_name);
                        }
                    }
                }
                GDSRecord::EndStr => {
                    state.require(ParserState::Structure, record_type)?;
                    if let Some(cell) = cell.take() {
                        library.cells.insert(cell.name().to_string(), cell);
                    }
                    state = ParserState::Library;
                }
                GDSRecord::EndLib => {
                    state.require(ParserState::Library, record_type)?;
                    return Ok(library);
                }
                GDSRecord::Boundary => {
                    state.require(ParserState::Structure, record_type)?;
                    property_attribute = None;
                    polygon = Some(Polygon::default());
                    state = ParserState::Element;
                }
                GDSRecord::Box => {
                    state.require(ParserState::Structure, record_type)?;
                    property_attribute = None;
                    gds_box = Some(GdsBox::default());
                    state = ParserState::Element;
                }
                GDSRecord::Node => {
                    state.require(ParserState::Structure, record_type)?;
                    property_attribute = None;
                    node = Some(Node::default());
                    state = ParserState::Element;
                }
                GDSRecord::Path => {
                    state.require(ParserState::Structure, record_type)?;
                    property_attribute = None;
                    path = Some(Path::default());
                    state = ParserState::Element;
                }
                GDSRecord::ARef | GDSRecord::SRef => {
                    state.require(ParserState::Structure, record_type)?;
                    property_attribute = None;
                    reference = Some(Reference::default());
                    state = ParserState::Element;
                }
                GDSRecord::Text => {
                    state.require(ParserState::Structure, record_type)?;
                    property_attribute = None;
                    text = Some(Text::default());
                    state = ParserState::Element;
                }
                GDSRecord::TextNode => {
                    state.require(ParserState::Structure, record_type)?;
                    state = ParserState::Element;
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
                    state.require(ParserState::Element, record_type)?;
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
                    state = ParserState::Structure;
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

    match state {
        ParserState::Element => Err(invalid_data("Unexpected EOF before ENDEL record")),
        ParserState::Structure => Err(invalid_data("Unexpected EOF before ENDSTR record")),
        ParserState::Library => Err(invalid_data("Unexpected EOF before ENDLIB record")),
    }
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
    payload: Vec<u8>,
}

impl<R: Read> RecordReader<R> {
    pub const fn new(reader: BufReader<R>) -> Self {
        Self {
            reader,
            payload: Vec::new(),
        }
    }

    fn read_payload(&mut self, payload_len: usize, record: GDSRecord) -> Result<&[u8], GdsError> {
        self.payload.resize(payload_len, 0);
        self.reader.read_exact(&mut self.payload).map_err(|error| {
            if error.kind() == io::ErrorKind::UnexpectedEof {
                invalid_data(format!("Truncated {record:?} record payload"))
            } else {
                GdsError::from(error)
            }
        })?;
        Ok(&self.payload)
    }
}

impl<R: Read> Iterator for RecordReader<R> {
    type Item = Result<(GDSRecord, GDSRecordData), GdsError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut header = [0u8; 4];
        let header_len = header.len();
        let buffered = self.reader.buffer();
        if buffered.len() >= header_len {
            header.copy_from_slice(&buffered[..header_len]);
            self.reader.consume(header_len);
        } else {
            let bytes_read = loop {
                match self.reader.read(&mut header) {
                    Ok(0) => return None,
                    Ok(bytes_read) => break bytes_read,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => return Some(Err(GdsError::from(error))),
                }
            };
            if bytes_read < header_len
                && let Err(error) = self.reader.read_exact(&mut header[bytes_read..])
            {
                return Some(Err(if error.kind() == io::ErrorKind::UnexpectedEof {
                    invalid_data("Truncated record header")
                } else {
                    GdsError::from(error)
                }));
            }
        }

        let size = u16::from_be_bytes([header[0], header[1]]) as usize;
        if size < 4 {
            return Some(Err(invalid_data(format!(
                "Record size {size} is smaller than the four-byte header"
            ))));
        }

        let Some(&layout) = RECORD_LAYOUTS.get(usize::from(header[2])) else {
            return Some(Err(invalid_data(format!(
                "Invalid record type byte: {:#04x}",
                header[2]
            ))));
        };
        let record = layout.record;
        let expected_data_type = layout.data_type;
        if header[3] != expected_data_type as u8 {
            return Some(Err(invalid_data(format!(
                "Invalid {record:?} data type: expected {expected_data_type:?}, found {:#04x}",
                header[3]
            ))));
        }
        let data_type = expected_data_type;
        let payload_len = size - 4;
        let expected_payload_len = layout.payload_len;
        if expected_payload_len != VARIABLE_PAYLOAD_LEN
            && payload_len != usize::from(expected_payload_len)
        {
            return Some(Err(invalid_data(format!(
                "Invalid {record:?} payload length: expected {expected_payload_len} bytes, found {payload_len}"
            ))));
        }

        let data = if payload_len == 0 {
            GDSRecordData::None
        } else if data_type == GDSDataType::AsciiString {
            let mut buf = vec![0u8; payload_len];
            if let Err(error) = self.reader.read_exact(&mut buf) {
                return Some(Err(if error.kind() == io::ErrorKind::UnexpectedEof {
                    invalid_data(format!("Truncated {record:?} record payload"))
                } else {
                    GdsError::from(error)
                }));
            }
            match String::from_utf8(buf) {
                Ok(mut result) => {
                    if !result.len().is_multiple_of(2) {
                        return Some(Err(misaligned_payload(record, result.len(), 2)));
                    }
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
            }
        } else {
            let buf = match self.read_payload(payload_len, record) {
                Ok(buf) => buf,
                Err(error) => return Some(Err(error)),
            };

            match data_type {
                GDSDataType::TwoByteSignedInteger | GDSDataType::BitArray => {
                    GDSRecordData::I16(read_i16_be(buf))
                }
                GDSDataType::FourByteSignedInteger | GDSDataType::FourByteReal => {
                    if expected_payload_len == VARIABLE_PAYLOAD_LEN {
                        let alignment = if record == GDSRecord::XY { 8 } else { 4 };
                        if !buf.len().is_multiple_of(alignment) {
                            return Some(Err(misaligned_payload(record, buf.len(), alignment)));
                        }
                    }
                    GDSRecordData::I32(read_i32_be(buf))
                }
                GDSDataType::EightByteReal => GDSRecordData::F64(
                    read_u64_be(buf)
                        .into_iter()
                        .map(eight_byte_real_to_float)
                        .collect(),
                ),
                GDSDataType::NoData | GDSDataType::AsciiString => {
                    return Some(Err(invalid_data(format!(
                        "Invalid {record:?} payload state"
                    ))));
                }
            }
        };

        Some(Ok((record, data)))
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RecordLayout {
    record: GDSRecord,
    data_type: GDSDataType,
    payload_len: u8,
}

const VARIABLE_PAYLOAD_LEN: u8 = u8::MAX;

const fn layout(record: GDSRecord, data_type: GDSDataType, payload_len: u8) -> RecordLayout {
    RecordLayout {
        record,
        data_type,
        payload_len,
    }
}

const RECORD_LAYOUTS: [RecordLayout; 60] = [
    layout(GDSRecord::Header, TwoByteSignedInteger, 2),
    layout(GDSRecord::BgnLib, TwoByteSignedInteger, 24),
    layout(GDSRecord::LibName, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::Units, EightByteReal, 16),
    layout(GDSRecord::EndLib, NoData, 0),
    layout(GDSRecord::BgnStr, TwoByteSignedInteger, 24),
    layout(GDSRecord::StrName, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::EndStr, NoData, 0),
    layout(GDSRecord::Boundary, NoData, 0),
    layout(GDSRecord::Path, NoData, 0),
    layout(GDSRecord::SRef, NoData, 0),
    layout(GDSRecord::ARef, NoData, 0),
    layout(GDSRecord::Text, NoData, 0),
    layout(GDSRecord::Layer, TwoByteSignedInteger, 2),
    layout(GDSRecord::DataType, TwoByteSignedInteger, 2),
    layout(GDSRecord::Width, FourByteSignedInteger, 4),
    layout(GDSRecord::XY, FourByteSignedInteger, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::EndEl, NoData, 0),
    layout(GDSRecord::SName, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::ColRow, TwoByteSignedInteger, 4),
    layout(GDSRecord::TextNode, NoData, 0),
    layout(GDSRecord::Node, NoData, 0),
    layout(GDSRecord::TextType, TwoByteSignedInteger, 2),
    layout(GDSRecord::Presentation, BitArray, 2),
    layout(
        GDSRecord::Spacing,
        FourByteSignedInteger,
        VARIABLE_PAYLOAD_LEN,
    ),
    layout(GDSRecord::String, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::STrans, BitArray, 2),
    layout(GDSRecord::Mag, EightByteReal, 8),
    layout(GDSRecord::Angle, EightByteReal, 8),
    layout(GDSRecord::UInteger, TwoByteSignedInteger, 2),
    layout(GDSRecord::UString, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::RefLibs, AsciiString, 88),
    layout(GDSRecord::Fonts, AsciiString, 176),
    layout(GDSRecord::PathType, TwoByteSignedInteger, 2),
    layout(GDSRecord::Generations, TwoByteSignedInteger, 2),
    layout(GDSRecord::AttrTable, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::StyTable, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::StrType, TwoByteSignedInteger, 2),
    layout(GDSRecord::ElFlags, BitArray, 2),
    layout(GDSRecord::ElKey, FourByteSignedInteger, 4),
    layout(GDSRecord::LinkType, TwoByteSignedInteger, 2),
    layout(
        GDSRecord::LinkKeys,
        FourByteSignedInteger,
        VARIABLE_PAYLOAD_LEN,
    ),
    layout(GDSRecord::NodeType, TwoByteSignedInteger, 2),
    layout(GDSRecord::PropAttr, TwoByteSignedInteger, 2),
    layout(GDSRecord::PropValue, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::Box, NoData, 0),
    layout(GDSRecord::BoxType, TwoByteSignedInteger, 2),
    layout(GDSRecord::Plex, FourByteSignedInteger, 4),
    layout(GDSRecord::BgnExtn, FourByteSignedInteger, 4),
    layout(GDSRecord::EndExtn, FourByteSignedInteger, 4),
    layout(GDSRecord::TapeNum, TwoByteSignedInteger, 2),
    layout(GDSRecord::TapeCode, TwoByteSignedInteger, 12),
    layout(GDSRecord::StrClass, BitArray, 2),
    layout(
        GDSRecord::Reserved,
        FourByteSignedInteger,
        VARIABLE_PAYLOAD_LEN,
    ),
    layout(GDSRecord::Format, TwoByteSignedInteger, 2),
    layout(GDSRecord::Mask, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::EndMasks, NoData, 0),
    layout(GDSRecord::LibDirSize, TwoByteSignedInteger, 2),
    layout(GDSRecord::SrfName, AsciiString, VARIABLE_PAYLOAD_LEN),
    layout(GDSRecord::LibSecure, TwoByteSignedInteger, 2),
];

#[cold]
fn misaligned_payload(record: GDSRecord, payload_len: usize, alignment: usize) -> GdsError {
    invalid_data(format!(
        "{record:?} payload length {payload_len} is not aligned to {alignment} bytes"
    ))
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
            assert!(
                matches!(
                    Library::from_bytes(&bytes, None),
                    Err(GdsError::InvalidData { .. })
                ),
                "{name}"
            );
        }
    }

    #[test]
    fn record_reader_distinguishes_clean_eof_from_partial_header() {
        let mut clean = RecordReader::new(BufReader::new(Cursor::new(Vec::<u8>::new())));
        assert!(clean.next().is_none());

        for bytes in [vec![0], vec![0, 4], vec![0, 4, GDSRecord::EndLib as u8]] {
            let mut reader = RecordReader::new(BufReader::new(Cursor::new(bytes)));
            assert!(matches!(
                reader.next(),
                Some(Err(GdsError::InvalidData { ref message }))
                    if message == "Truncated record header"
            ));
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
        let mut missing_end_element = missing_end_structure.clone();
        missing_end_element.extend_from_slice(&record(
            GDSRecord::Boundary,
            GDSDataType::NoData,
            &[],
        ));

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
    fn fixed_and_string_record_grammar_is_enforced() {
        for (record_type, data_type, payload_len) in [
            (GDSRecord::Generations, GDSDataType::TwoByteSignedInteger, 2),
            (GDSRecord::ElFlags, GDSDataType::BitArray, 2),
            (GDSRecord::Plex, GDSDataType::FourByteSignedInteger, 4),
            (GDSRecord::StrType, GDSDataType::TwoByteSignedInteger, 2),
            (GDSRecord::TapeNum, GDSDataType::TwoByteSignedInteger, 2),
            (GDSRecord::TapeCode, GDSDataType::TwoByteSignedInteger, 12),
        ] {
            let bytes = [
                record(record_type, data_type, &vec![0; payload_len]),
                record(GDSRecord::EndLib, GDSDataType::NoData, &[]),
            ]
            .concat();
            assert!(Library::from_bytes(&bytes, None).is_ok(), "{record_type:?}");

            let bytes = record(record_type, data_type, &[]);
            assert!(
                matches!(
                    Library::from_bytes(&bytes, None),
                    Err(GdsError::InvalidData { .. })
                ),
                "{record_type:?}"
            );
        }

        for record_type in [
            GDSRecord::LibName,
            GDSRecord::StrName,
            GDSRecord::SName,
            GDSRecord::String,
            GDSRecord::PropValue,
        ] {
            let bytes = record(record_type, GDSDataType::TwoByteSignedInteger, &[0, 0]);
            assert!(
                matches!(
                    Library::from_bytes(&bytes, None),
                    Err(GdsError::InvalidData { .. })
                ),
                "{record_type:?}"
            );

            let bytes = record(record_type, GDSDataType::AsciiString, b"x");
            assert!(
                matches!(
                    Library::from_bytes(&bytes, None),
                    Err(GdsError::InvalidData { .. })
                ),
                "{record_type:?}"
            );
        }
    }

    #[test]
    fn fixed_library_string_lengths_are_enforced() {
        for (record_type, expected_payload_len) in
            [(GDSRecord::RefLibs, 88), (GDSRecord::Fonts, 176)]
        {
            for (payload_len, is_valid) in [
                (expected_payload_len, true),
                (expected_payload_len - 2, false),
                (expected_payload_len + 2, false),
            ] {
                let bytes = [
                    record(record_type, GDSDataType::AsciiString, &vec![0; payload_len]),
                    record(GDSRecord::EndLib, GDSDataType::NoData, &[]),
                ]
                .concat();
                assert_eq!(
                    Library::from_bytes(&bytes, None).is_ok(),
                    is_valid,
                    "{record_type:?} payload length {payload_len}"
                );
            }
        }
    }

    #[test]
    fn invalid_structural_sequences_are_rejected() {
        let begin_structure = record(
            GDSRecord::BgnStr,
            GDSDataType::TwoByteSignedInteger,
            &[0; 24],
        );
        let begin_boundary = record(GDSRecord::Boundary, GDSDataType::NoData, &[]);
        let begin_path = record(GDSRecord::Path, GDSDataType::NoData, &[]);

        for records in [
            vec![begin_structure.clone(), begin_structure.clone()],
            vec![begin_structure.clone(), begin_boundary, begin_path],
            vec![record(GDSRecord::EndEl, GDSDataType::NoData, &[])],
            vec![
                begin_structure,
                record(GDSRecord::EndLib, GDSDataType::NoData, &[]),
            ],
        ] {
            let bytes = records.concat();
            assert_invalid(&Library::from_bytes(&bytes, None));
        }
    }

    #[test]
    fn end_library_ignores_trailing_padding_and_records() {
        let bytes = Library::new("minimal")
            .to_bytes_with_timestamp_policy(1e-3, 1e-9, GdsTimestampPolicy::Zero)
            .expect("minimal library should serialize");
        let expected = Library::from_bytes(&bytes, None).expect("minimal library should parse");

        for suffix in [
            vec![0],
            record(
                GDSRecord::BgnStr,
                GDSDataType::TwoByteSignedInteger,
                &[0; 24],
            ),
        ] {
            let mut padded = bytes.clone();
            padded.extend_from_slice(&suffix);
            let parsed = Library::from_bytes(&padded, None)
                .expect("bytes after ENDLIB should not be semantically parsed");
            assert_eq!(parsed, expected);
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
    fn generated_record_sequence_never_panics(mut records: Vec<(u8, u8, Vec<u8>)>) -> bool {
        records.truncate(16);
        let mut bytes = Vec::new();
        for (record_type, data_type, mut payload) in records {
            payload.truncate(256);
            let size = 4 + payload.len() as u16;
            bytes.extend_from_slice(&size.to_be_bytes());
            bytes.push(record_type);
            bytes.push(data_type);
            bytes.extend_from_slice(&payload);
        }
        bytes.extend_from_slice(&[0, 4, GDSRecord::EndLib as u8, GDSDataType::NoData as u8]);

        std::panic::catch_unwind(|| Library::from_bytes(&bytes, None)).is_ok()
    }
}
