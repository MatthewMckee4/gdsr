use bytemuck::cast_slice;
use chrono::{Datelike, Local, Timelike};
use pyo3::prelude::*;
use std::fs::File;
use std::io::Write;

use crate::{
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

    let len = library_name.len() + if library_name.len() % 2 != 0 { 1 } else { 0 };

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
        (4 + len) as u16,
        combine_record_and_data_type(GDSRecord::LibName, GDSDataType::AsciiString),
    ];

    write_u16_array_to_file(&mut head_start, &mut file)?;
    file.write_all(library_name.as_bytes())?;

    let mut head_units = [
        20,
        combine_record_and_data_type(GDSRecord::Units, GDSDataType::EightByteReal),
    ];
    write_u16_array_to_file(&mut head_units, &mut file)?;

    let units = [
        eight_byte_real(precision / units),
        eight_byte_real(precision),
    ];

    for unit in &units {
        file.write_all(unit)?;
    }

    Ok(file)
}

pub fn write_gds_tail_to_file(mut file: File) -> PyResult<File> {
    let mut tail = [
        4,
        combine_record_and_data_type(GDSRecord::EndLib, GDSDataType::NoData),
    ];
    write_u16_array_to_file(&mut tail, &mut file)?;

    Ok(file)
}

pub fn write_u16_array_to_file(array: &mut [u16], file: &mut File) -> PyResult<()> {
    u16_array_to_big_endian(array);
    file.write_all(cast_slice(array))?;

    Ok(())
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
    write_u16_array_to_file(&mut tail, &mut file)?;

    Ok(file)
}
