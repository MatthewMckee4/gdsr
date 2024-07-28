use std::fs::File;
use std::vec;

use chrono::{Datelike, Local, Timelike};

use pyo3::prelude::*;

use crate::config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord};
use crate::utils::io::{write_gds, write_string_with_record_to_file, write_u16_array_to_file};

use super::*;

impl Cell {
    pub fn _to_gds(&self, mut file: File, units: f64, precision: f64) -> PyResult<File> {
        let now = Local::now();
        let timestamp = now.naive_utc();

        let mut cell_head = [
            28,
            combine_record_and_data_type(GDSRecord::BgnStr, GDSDataType::TwoByteSignedInteger),
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

        file = write_u16_array_to_file(file, &mut cell_head)?;

        file = write_string_with_record_to_file(file, GDSRecord::StrName, &self.name)?;

        for path in &self.paths {
            file = path._to_gds(file, units / precision)?;
        }

        for polygon in &self.polygons {
            file = polygon._to_gds(file, units / precision)?;
        }

        for text in &self.texts {
            file = text._to_gds(file, units / precision)?;
        }

        for cell_reference in &self.cell_references {
            file = cell_reference._to_gds(file, units / precision)?;
        }

        let mut cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        file = write_u16_array_to_file(file, &mut cell_tail)?;

        Ok(file)
    }
}

#[pymethods]
impl Cell {
    #[pyo3(signature=(file_name, units=1e-6, precision=1e-10))]
    pub fn to_gds(&self, file_name: &str, units: f64, precision: f64) -> PyResult<()> {
        write_gds(file_name, "library", units, precision, vec![self.clone()])
    }
}
