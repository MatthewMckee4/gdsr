use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

use bytemuck::cast_slice;
use byteorder::{BigEndian, ByteOrder};
use chrono::{Datelike, Local, Timelike};

use log::{debug, info};
use pyo3::{exceptions::PyIOError, prelude::*};
use tempfile::Builder;

use crate::cell::Cell;
use crate::config::gds_file_types::GDSRecordData;
use crate::config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord};
use crate::library::Library;
use crate::path::path_type::PathType;
use crate::path::Path;
use crate::point::{get_points_from_i32_vec, Point};
use crate::polygon::Polygon;
use crate::reference::{Reference, ReferenceInstance};
use crate::text::utils::get_presentations_from_value;
use crate::text::Text;

use super::gds_format::{eight_byte_real, u16_array_to_big_endian};

pub fn write_gds_head_to_file(
    library_name: &str,
    units: f64,
    precision: f64,
    mut file: File,
) -> PyResult<File> {
    let now = Local::now();
    let timestamp = now.naive_utc();

    let mut head_start = [
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

    file = write_u16_array_to_file(file, &mut head_start)?;

    file = write_string_with_record_to_file(file, GDSRecord::LibName, library_name)?;

    let mut head_units = [
        20,
        combine_record_and_data_type(GDSRecord::Units, GDSDataType::EightByteReal),
    ];
    file = write_u16_array_to_file(file, &mut head_units)?;

    file = write_float_to_eight_byte_real_to_file(file, precision / units)?;
    file = write_float_to_eight_byte_real_to_file(file, precision)?;

    Ok(file)
}

pub fn write_gds_tail_to_file(mut file: File) -> PyResult<File> {
    let mut tail = [
        4,
        combine_record_and_data_type(GDSRecord::EndLib, GDSDataType::NoData),
    ];
    file = write_u16_array_to_file(file, &mut tail)?;

    Ok(file)
}

pub fn write_u16_array_to_file(mut file: File, array: &mut [u16]) -> PyResult<File> {
    u16_array_to_big_endian(array);
    file.write_all(cast_slice(array))?;

    Ok(file)
}

pub fn write_float_to_eight_byte_real_to_file(mut file: File, value: f64) -> PyResult<File> {
    let value = eight_byte_real(value);
    file.write_all(&value)?;

    Ok(file)
}

pub fn write_points_to_file(mut file: File, points: &[Point], scale: f64) -> PyResult<File> {
    let xy_head = combine_record_and_data_type(GDSRecord::XY, GDSDataType::FourByteSignedInteger);
    let points_length = points.len();
    let mut i0 = 0;

    let mut xy_header_buffer = vec![0u8; 4];
    let mut points_buffer = Vec::with_capacity(8 * 8190);

    while i0 < points_length {
        let i1 = (i0 + 8190).min(points_length);
        let record_length = 4 + 8 * (i1 - i0);

        xy_header_buffer[0..2].copy_from_slice(&(record_length as u16).to_be_bytes());
        xy_header_buffer[2..4].copy_from_slice(&xy_head.to_be_bytes());

        file.write_all(&xy_header_buffer)?;

        points_buffer.clear();
        for point in &points[i0..i1] {
            let scaled_x = (point.x * scale).round() as i32;
            let scaled_y = (point.y * scale).round() as i32;

            points_buffer.extend_from_slice(&scaled_x.to_be_bytes());
            points_buffer.extend_from_slice(&scaled_y.to_be_bytes());
        }

        file.write_all(&points_buffer)?;

        i0 = i1;
    }

    Ok(file)
}

pub fn write_element_tail_to_file(mut file: File) -> PyResult<File> {
    let mut tail = [
        4,
        combine_record_and_data_type(GDSRecord::EndEl, GDSDataType::NoData),
    ];
    file = write_u16_array_to_file(file, &mut tail)?;

    Ok(file)
}

pub fn write_string_with_record_to_file(
    mut file: File,
    record: GDSRecord,
    string: &str,
) -> PyResult<File> {
    let mut len = string.len();
    if len % 2 != 0 {
        len += 1;
    }

    let mut lib_name_bytes = string.as_bytes().to_vec();
    if string.len() % 2 != 0 {
        lib_name_bytes.push(0);
    }
    let mut string_start = [
        (4 + len) as u16,
        combine_record_and_data_type(record, GDSDataType::AsciiString),
    ];

    file = write_u16_array_to_file(file, &mut string_start)?;

    file.write_all(&lib_name_bytes)?;

    Ok(file)
}

pub fn write_gds(
    file_name: String,
    library_name: &str,
    units: f64,
    precision: f64,
    cells: HashMap<String, Cell>,
) -> PyResult<String> {
    let mut file = File::create(file_name.clone())
        .map_err(|_| PyIOError::new_err("Could not open file for writing"))?;

    file = write_gds_head_to_file(library_name, units, precision, file)?;

    let mut written_cell_names: HashSet<String> = HashSet::new();

    for (cell_name, cell) in &cells {
        if !written_cell_names.contains(cell_name) {
            written_cell_names.insert(cell_name.clone());
            file = cell._to_gds(file, units, precision, &mut written_cell_names)?;
        }
    }

    file = write_gds_tail_to_file(file)?;

    file.flush()?;

    Ok(file_name)
}

pub fn write_transformation_to_file(
    mut file: File,
    angle: f64,
    magnification: f64,
    x_reflection: bool,
) -> PyResult<File> {
    let transform_applied = angle != 0.0 || magnification != 1.0 || x_reflection;
    if transform_applied {
        let mut buffer_flags = [
            6,
            combine_record_and_data_type(GDSRecord::STrans, GDSDataType::BitArray),
            if x_reflection { 0x8000 } else { 0x0000 },
        ];

        file = write_u16_array_to_file(file, &mut buffer_flags)?;

        if magnification != 1.0 {
            let mut buffer_mag = [
                12,
                combine_record_and_data_type(GDSRecord::Mag, GDSDataType::EightByteReal),
            ];
            file = write_u16_array_to_file(file, &mut buffer_mag)?;
            file = write_float_to_eight_byte_real_to_file(file, magnification)?;
        }

        if angle != 0.0 {
            let mut buffer_rot = [
                12,
                combine_record_and_data_type(GDSRecord::Angle, GDSDataType::EightByteReal),
            ];
            file = write_u16_array_to_file(file, &mut buffer_rot)?;
            file = write_float_to_eight_byte_real_to_file(file, angle)?;
        }
    }

    Ok(file)
}

pub fn from_gds(file_name: String) -> PyResult<Library> {
    let mut library = Library::new("Library".to_string());

    let file = File::open(file_name)?;
    let reader = RecordReader::new(BufReader::new(file));

    let mut cell: Option<Cell> = None;
    let mut path: Option<Path> = None;
    let mut polygon: Option<Polygon> = None;
    let mut text: Option<Text> = None;
    let mut reference: Option<Reference> = None;

    let mut scale = 1.0;
    let mut rounding_digits = 0;

    for record in reader {
        match record {
            Ok((record_type, data)) => match record_type {
                GDSRecord::Header | GDSRecord::BgnLib => {}
                GDSRecord::LibName => {
                    debug!("LibName {:?}", data);
                    if let GDSRecordData::Str(name) = data {
                        library.name = name;
                    };
                    continue;
                }
                GDSRecord::Units => {
                    debug!("Units {:?}", data);
                    if let GDSRecordData::F64(units) = data {
                        scale = units[0];
                        rounding_digits = -(units[1] / units[0]).log10() as u32 - 1;
                    };
                    continue;
                }
                GDSRecord::EndLib => {
                    update_references(&mut library);
                    continue;
                }
                GDSRecord::BgnStr => {
                    debug!("BgnStr {:?}", data);
                    cell = Some(Cell::default());
                    continue;
                }
                GDSRecord::StrName => {
                    debug!("StrName {:?}", data);
                    if let GDSRecordData::Str(cell_name) = data {
                        if let Some(cell) = &mut cell {
                            cell.name = cell_name;
                        }
                    };
                    continue;
                }
                GDSRecord::EndStr => {
                    debug!("EndStr {:?}", data);
                    if let Some(cell) = &mut cell {
                        library.cells.insert(cell.name.clone(), cell.clone());
                    }
                    cell = None;
                    continue;
                }
                GDSRecord::Boundary | GDSRecord::Box => {
                    debug!("Boundary {:?}", data);
                    polygon = Some(Polygon::default());
                    continue;
                }
                GDSRecord::Path | GDSRecord::RaithMbmsPath => {
                    debug!("Path {:?}", data);
                    path = Some(Path::default());
                    continue;
                }
                GDSRecord::ARef | GDSRecord::SRef => {
                    debug!("ARef {:?}", data);
                    reference = Some(Reference::default());
                    continue;
                }
                GDSRecord::Text => {
                    debug!("Text {:?}", data);
                    text = Some(Text::default());
                    continue;
                }
                GDSRecord::Layer => {
                    debug!("Layer {:?}", data);
                    if let GDSRecordData::I16(layer) = data {
                        if let Some(polygon) = &mut polygon {
                            polygon.layer = layer[0] as i32;
                        } else if let Some(path) = &mut path {
                            path.layer = layer[0] as i32;
                        } else if let Some(text) = &mut text {
                            text.layer = layer[0] as i32;
                        }
                    };
                    continue;
                }
                GDSRecord::DataType | GDSRecord::BoxType => {
                    debug!("DataType {:?}", data);
                    if let GDSRecordData::I16(data_type) = data {
                        if let Some(polygon) = &mut polygon {
                            polygon.data_type = data_type[0] as i32;
                        } else if let Some(path) = &mut path {
                            path.data_type = data_type[0] as i32;
                        }
                    };
                    continue;
                }
                GDSRecord::Width => {
                    debug!("Width {:?}", data);
                    if let GDSRecordData::I32(width) = data {
                        if let Some(path) = &mut path {
                            path.width = Some(width[0] as f64 * scale);
                        }
                    };
                    continue;
                }
                GDSRecord::XY => {
                    debug!("XY {:?}", data);
                    if let GDSRecordData::I32(xy) = data {
                        let points = get_points_from_i32_vec(xy);
                        let points = points
                            .iter()
                            .map(|p| p.scale(scale, Point::default()))
                            .collect();
                        if let Some(polygon) = &mut polygon {
                            polygon.points = points;
                        } else if let Some(path) = &mut path {
                            path.points = points;
                        } else if let Some(reference) = &mut reference {
                            if points.len() == 1 {
                                reference.grid.origin = points[0];
                            } else if points.len() == 3 {
                                let points: Vec<Point> = points
                                    .iter()
                                    .map(|&p| p.rotate(-reference.grid.angle, points[0]))
                                    .collect();
                                reference.grid.origin = points[0];
                                reference.grid.spacing_x = if reference.grid.columns > 0 {
                                    ((points[1] - points[0]) / reference.grid.columns as f64)
                                        .round(rounding_digits)
                                } else {
                                    Point::default()
                                };
                                reference.grid.spacing_y = if reference.grid.rows > 0 {
                                    ((points[2] - points[0]) / reference.grid.rows as f64)
                                        .round(rounding_digits)
                                } else {
                                    Point::default()
                                };
                            }
                        } else if let Some(text) = &mut text {
                            text.origin = points[0];
                        }
                    };
                    continue;
                }
                GDSRecord::EndEl => {
                    debug!("EndEl {:?}", data);
                    if let Some(cell) = &mut cell {
                        if let Some(polygon) = &mut polygon {
                            cell.polygons.push(polygon.clone());
                        } else if let Some(path) = &mut path {
                            cell.paths.push(path.clone());
                        } else if let Some(text) = &mut text {
                            cell.texts.push(text.clone());
                        } else if let Some(reference) = &mut reference {
                            cell.references.push(reference.clone());
                        }
                        polygon = None;
                        path = None;
                        text = None;
                        reference = None;
                    };
                    continue;
                }
                GDSRecord::SName => {
                    debug!("SName {:?}", data);
                    if let GDSRecordData::Str(cell_name) = data {
                        if let Some(reference) = &mut reference {
                            match &mut reference.instance {
                                ReferenceInstance::Cell(cell) => {
                                    cell.name = cell_name;
                                }
                                ReferenceInstance::Element(_) => {}
                            }
                        }
                    };
                    continue;
                }
                GDSRecord::ColRow => {
                    debug!("ColRow {:?}", data);
                    if let GDSRecordData::I16(col_row) = data {
                        if let Some(reference) = &mut reference {
                            reference.grid.columns = col_row[0] as usize;
                            reference.grid.rows = col_row[1] as usize;
                        }
                    };
                    continue;
                }
                GDSRecord::TextNode => {
                    debug!("TextNode {}", data);
                    continue;
                }
                GDSRecord::Node => {
                    debug!("Node {}", data);
                    continue;
                }
                GDSRecord::TextType => {
                    debug!("TextType {:?}", data);
                    continue;
                }
                GDSRecord::Presentation => {
                    debug!("Presentation {:?}", data);
                    if let GDSRecordData::U16(flags) = data {
                        if let Some(text) = &mut text {
                            (text.vertical_presentation, text.horizontal_presentation) =
                                get_presentations_from_value(flags[0])?;
                        }
                    };
                    continue;
                }
                GDSRecord::Spacing => {
                    debug!("Spacing {}", data);
                    continue;
                }
                GDSRecord::String => {
                    debug!("String {:?}", data);
                    if let GDSRecordData::Str(string) = data {
                        if let Some(text) = &mut text {
                            text.text = string;
                        }
                    };
                    continue;
                }
                GDSRecord::STrans => {
                    debug!("STrans {:?}", data);
                    if let GDSRecordData::U16(flags) = data {
                        let x_reflection = flags[0] & 0x8000 != 0;
                        if let Some(text) = &mut text {
                            text.x_reflection = x_reflection;
                        }
                        if let Some(reference) = &mut reference {
                            reference.grid.x_reflection = x_reflection;
                        }
                    };
                    continue;
                }
                GDSRecord::Mag => {
                    debug!("Mag {:?}", data);
                    if let GDSRecordData::F64(magnification) = data {
                        if let Some(text) = &mut text {
                            text.magnification = magnification[0];
                        } else if let Some(reference) = &mut reference {
                            reference.grid.magnification = magnification[0];
                        }
                    };
                    continue;
                }
                GDSRecord::Angle => {
                    debug!("Angle {:?}", data);
                    if let GDSRecordData::F64(angle) = data {
                        if let Some(text) = &mut text {
                            text.angle = angle[0];
                        } else if let Some(reference) = &mut reference {
                            reference.grid.angle = angle[0];
                        }
                    };
                    continue;
                }
                GDSRecord::UInteger => {
                    debug!("UInteger {}", data);
                    continue;
                }
                GDSRecord::UString => {
                    debug!("UString {}", data);
                    continue;
                }
                GDSRecord::RefLibs => {
                    debug!("RefLibs {}", data);
                    continue;
                }
                GDSRecord::Fonts => {
                    debug!("Fonts {}", data);
                    continue;
                }
                GDSRecord::PathType => {
                    debug!("PathType {:?}", data);
                    if let GDSRecordData::I16(path_type) = data {
                        if let Some(path) = &mut path {
                            path.path_type = Some(PathType::new(path_type[0] as i32)?);
                        }
                    };
                    continue;
                }
                GDSRecord::Generations => {
                    debug!("Generations {}", data);
                    continue;
                }
                GDSRecord::AttrTable => {
                    debug!("AttrTable {}", data);
                    continue;
                }
                GDSRecord::StyTable => {
                    debug!("StyTable {}", data);
                    continue;
                }
                GDSRecord::StrType => {
                    debug!("StrType {}", data);
                    continue;
                }
                GDSRecord::ElFlags => {
                    debug!("ElFlags {}", data);
                    continue;
                }
                GDSRecord::ElKey => {
                    debug!("ElKey {}", data);
                    continue;
                }
                GDSRecord::LinkType => {
                    debug!("LinkType {}", data);
                    continue;
                }
                GDSRecord::LinkKeys => {
                    debug!("LinkKeys {}", data);
                    continue;
                }
                GDSRecord::NodeType => {
                    debug!("NodeType {}", data);
                    continue;
                }
                GDSRecord::PropAttr => {
                    debug!("PropAttr {}", data);
                    continue;
                }
                GDSRecord::PropValue => {
                    debug!("PropValue {}", data);
                    continue;
                }
                GDSRecord::Plex => {
                    debug!("Plex {}", data);
                    continue;
                }
                GDSRecord::BgnExtn => {
                    debug!("BgnExtn {}", data);
                    continue;
                }
                GDSRecord::EndExtn => {
                    debug!("EndExtn {}", data);
                    continue;
                }
                GDSRecord::TapeNum => {
                    debug!("TapeNum {}", data);
                    continue;
                }
                GDSRecord::TapeCode => {
                    debug!("TapeCode {}", data);
                    continue;
                }
                GDSRecord::StrClass => {
                    debug!("StrClass {}", data);
                    continue;
                }
                GDSRecord::Reserved => {
                    debug!("Reserved {}", data);
                    continue;
                }
                GDSRecord::Format => {
                    debug!("Format {}", data);
                    continue;
                }
                GDSRecord::Mask => {
                    debug!("Mask {}", data);
                    continue;
                }
                GDSRecord::EndMasks => {
                    debug!("EndMasks {}", data);
                    continue;
                }
                GDSRecord::LibDirSize => {
                    debug!("LibDirSize {}", data);
                    continue;
                }
                GDSRecord::SrfName => {
                    debug!("SrfName {}", data);
                    continue;
                }
                GDSRecord::LibSecur => {
                    debug!("LibSecur {}", data);
                    continue;
                }
                GDSRecord::RaithPxxData => {
                    debug!("RaithPxxData {}", data);
                    continue;
                }
            },
            Err(e) => return Err(e),
        }
    }

    Ok(library)
}

fn update_references(library: &mut Library) {
    let cells = &mut library.cells;
    let cells_cloned = cells.clone();
    for cell in cells.values_mut() {
        for reference in cell.references.iter_mut() {
            match &reference.instance {
                ReferenceInstance::Cell(cell) => {
                    if let Some(referenced_cell) = cells_cloned.get(&cell.name) {
                        reference.instance = ReferenceInstance::Cell(referenced_cell.clone());
                    } else {
                        info!("Cell {} not found", cell.name);
                    }
                }
                ReferenceInstance::Element(_) => {}
            }
        }
    }
}

pub struct RecordReader<R: Read> {
    reader: BufReader<R>,
}

impl<R: Read> RecordReader<R> {
    pub fn new(reader: BufReader<R>) -> Self {
        RecordReader { reader }
    }
}

impl<R: Read> Iterator for RecordReader<R> {
    type Item = PyResult<(GDSRecord, GDSRecordData)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut header = [0u8; 4];
        if let Err(e) = self.reader.read_exact(&mut header) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                return None;
            } else {
                return Some(Err(PyErr::from(e)));
            }
        }

        let size = u16::from_be_bytes([header[0], header[1]]) as usize;
        let record_type = header[2];
        let data_type = header[3];

        let data = if size > 4 {
            match GDSDataType::try_from(data_type) {
                Ok(data_type) => match data_type {
                    GDSDataType::BitArray => {
                        let mut buf = vec![0u8; size - 4];
                        if let Err(e) = self.reader.read_exact(&mut buf) {
                            return Some(Err(PyErr::from(e)));
                        }
                        let mut result = Vec::with_capacity(buf.len() / 2);
                        for chunk in buf.chunks_exact(2) {
                            let value = BigEndian::read_u16(chunk);
                            result.push(value);
                        }
                        GDSRecordData::U16(result)
                    }
                    GDSDataType::TwoByteSignedInteger => {
                        let mut buf = vec![0u8; size - 4];
                        if let Err(e) = self.reader.read_exact(&mut buf) {
                            return Some(Err(PyErr::from(e)));
                        }
                        let mut result = Vec::with_capacity(buf.len() / 2);
                        for chunk in buf.chunks_exact(2) {
                            let value = BigEndian::read_i16(chunk);
                            result.push(value);
                        }
                        GDSRecordData::I16(result)
                    }
                    GDSDataType::FourByteSignedInteger => {
                        let mut buf = vec![0u8; size - 4];
                        if let Err(e) = self.reader.read_exact(&mut buf) {
                            return Some(Err(PyErr::from(e)));
                        }
                        let mut result = Vec::with_capacity(buf.len() / 4);
                        for chunk in buf.chunks_exact(4) {
                            let value = BigEndian::read_i32(chunk);
                            result.push(value);
                        }
                        GDSRecordData::I32(result)
                    }
                    GDSDataType::EightByteReal => {
                        let mut buf = vec![0u8; size - 4];
                        if let Err(e) = self.reader.read_exact(&mut buf) {
                            return Some(Err(PyErr::from(e)));
                        }
                        let mut result = Vec::with_capacity(buf.len() / 8);
                        for chunk in buf.chunks_exact(8) {
                            let value = eight_byte_real_to_float(BigEndian::read_u64(chunk));
                            result.push(value);
                        }
                        GDSRecordData::F64(result)
                    }
                    GDSDataType::AsciiString => {
                        let mut buf = vec![0u8; size - 4];
                        if let Err(e) = self.reader.read_exact(&mut buf) {
                            return Some(Err(PyErr::from(e)));
                        }
                        let data = if buf.last() == Some(&0) {
                            buf.pop();
                            String::from_utf8_lossy(&buf).into_owned()
                        } else {
                            String::from_utf8_lossy(&buf).into_owned()
                        };
                        GDSRecordData::Str(data)
                    }
                    _ => {
                        let mut buf = vec![0u8; size - 4];
                        if let Err(e) = self.reader.read_exact(&mut buf) {
                            return Some(Err(PyErr::from(e)));
                        }
                        GDSRecordData::Str(String::from_utf8_lossy(&buf).into_owned())
                    }
                },
                Err(_) => GDSRecordData::None,
            }
        } else {
            GDSRecordData::None
        };

        let record = match GDSRecord::try_from(record_type) {
            Ok(record) => record,
            Err(_) => {
                return Some(Err(PyIOError::new_err(format!(
                    "Invalid record type: {}",
                    record_type
                ))));
            }
        };

        Some(Ok((record, data)))
    }
}

fn eight_byte_real_to_float(bytes: u64) -> f64 {
    let short1 = (bytes >> 48) as u16;
    let short2 = ((bytes >> 32) & 0xFFFF) as u16;
    let long3 = (bytes & 0xFFFFFFFF) as u32;

    let exponent = ((short1 & 0x7F00) >> 8) as i32 - 64;
    let mantissa = (((short1 & 0x00FF) as u64 * 65536 + short2 as u64) * 4294967296 + long3 as u64)
        as f64
        / 72057594037927936.0;

    if short1 & 0x8000 != 0 {
        -mantissa * 16.0_f64.powi(exponent)
    } else {
        mantissa * 16.0_f64.powi(exponent)
    }
}

pub fn create_temp_file() -> PyResult<String> {
    let temp_file = Builder::new().suffix(".gds").tempfile()?;
    let temp_path = temp_file.path().to_string_lossy().to_string();
    Ok(temp_path)
}
