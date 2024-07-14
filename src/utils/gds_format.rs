use bytemuck::cast_slice;
use chrono::{Datelike, Local, Timelike};
use log::info;
use pyo3::prelude::*;
use std::fs::File;
use std::io::Write;

use crate::config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord};

pub fn eight_byte_real(value: f64) -> [u8; 8] {
    if value == 0.0 {
        return [0x00; 8];
    }

    let mut byte1: u8;
    let mut val = value;

    if val < 0.0 {
        byte1 = 0x80;
        val = -val;
    } else {
        byte1 = 0x00;
    }

    let fexp = val.log2() / 4.0;
    let mut exponent = fexp.ceil() as i32;

    if fexp == exponent as f64 {
        exponent += 1;
    }

    let mantissa = (val * 16.0_f64.powi(14 - exponent)) as u64;
    byte1 += (exponent + 64) as u8;

    let byte2 = (mantissa >> 48) as u8;
    let short3 = ((mantissa >> 32) & 0xFFFF) as u16;
    let long4 = (mantissa & 0xFFFFFFFF) as u32;

    let mut result = [0u8; 8];
    result[0] = byte1;
    result[1] = byte2;
    result[2] = (short3 >> 8) as u8;
    result[3] = (short3 & 0xFF) as u8;
    result[4] = (long4 >> 24) as u8;
    result[5] = (long4 >> 16) as u8;
    result[6] = (long4 >> 8) as u8;
    result[7] = (long4 & 0xFF) as u8;

    result
}

pub fn u16_array_to_big_endian(array: &mut [u16]) {
    for value in array.iter_mut() {
        *value = value.to_be();
    }
}

pub fn i32_array_to_big_endian(array: &mut [i32]) {
    for value in array.iter_mut() {
        *value = value.to_be();
    }
}

pub fn write_u16_array_to_file(array: &mut [u16], file: &mut File) -> PyResult<()> {
    u16_array_to_big_endian(array);
    file.write_all(cast_slice(array))?;

    Ok(())
}

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
