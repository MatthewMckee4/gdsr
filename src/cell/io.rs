use std::fs::File;
use std::io::Write;

use chrono::{Datelike, Local, Timelike};

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;

use crate::config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord};
use crate::utils::io::{write_gds_head_to_file, write_gds_tail_to_file, write_u16_array_to_file};

use super::*;

impl Cell {
    pub fn _to_gds(&self, mut file: File, units: f64, precision: f64) -> PyResult<File> {
        let now = Local::now();
        let timestamp = now.naive_utc();

        let len = self.name.len() + if self.name.len() % 2 != 0 { 1 } else { 0 };

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
            (4 + len) as u16,
            combine_record_and_data_type(GDSRecord::StrName, GDSDataType::AsciiString),
        ];

        write_u16_array_to_file(&mut cell_head, &mut file)?;

        file.write_all(self.name.as_bytes())?;

        for path in &self.paths {
            file = path._to_gds(file, units / precision)?;
        }

        for polygon in &self.polygons {
            file = polygon._to_gds(file, units / precision)?;
        }

        let mut cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        write_u16_array_to_file(&mut cell_tail, &mut file)?;

        Ok(file)
    }
}

#[pymethods]
impl Cell {
    #[pyo3(signature=(file_name, units=1e-6, precision=1e-10))]
    pub fn to_gds(&self, file_name: &str, units: f64, precision: f64) -> PyResult<()> {
        let mut file = File::create(file_name)
            .map_err(|_| PyIOError::new_err("Could not open file for writing"))?;

        file = write_gds_head_to_file(&self.name, units, precision, file)?;

        file = self._to_gds(file, units, precision)?;

        file = write_gds_tail_to_file(file)?;

        file.flush()?;

        Ok(())
    }
}
