use std::collections::{HashMap, HashSet};

use std::fs::File;
use std::io::Write;

use bytemuck::cast_slice;
use chrono::{Datelike, Local, Timelike};
use pyo3::{exceptions::PyIOError, prelude::*};

use crate::{
    cell::Cell,
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    point::Point,
};

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

    file = write_eight_byte_real_to_file(file, precision / units)?;
    file = write_eight_byte_real_to_file(file, precision)?;

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

pub fn write_eight_byte_real_to_file(mut file: File, value: f64) -> PyResult<File> {
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
    file_name: &str,
    library_name: &str,
    units: f64,
    precision: f64,
    cells: HashMap<String, Cell>,
) -> PyResult<()> {
    let mut file = File::create(file_name)
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

    Ok(())
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
            file = write_eight_byte_real_to_file(file, magnification)?;
        }

        if angle != 0.0 {
            let mut buffer_rot = [
                12,
                combine_record_and_data_type(GDSRecord::Angle, GDSDataType::EightByteReal),
            ];
            file = write_u16_array_to_file(file, &mut buffer_rot)?;
            file = write_eight_byte_real_to_file(file, angle)?;
        }
    }

    Ok(file)
}
