use chrono::{Datelike, Local, Timelike};

use crate::Cell;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_structure_name, write_string_with_record_to_file, write_u16_array_to_file,
};

impl ToGds for Cell {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        use rayon::prelude::*;

        validate_structure_name(&self.name)?;

        let now = Local::now();
        let timestamp = now.naive_utc();

        let cell_head = [
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

        let mut buffer = Vec::new();

        write_u16_array_to_file(&mut buffer, &cell_head)?;

        write_string_with_record_to_file(&mut buffer, GDSRecord::StrName, &self.name)?;

        let path_bufs: Result<Vec<_>, _> = self
            .paths
            .par_iter()
            .map(|p| p.to_gds_impl(database_units))
            .collect();
        for b in path_bufs? {
            buffer.extend_from_slice(&b);
        }

        let polygon_bufs: Result<Vec<_>, _> = self
            .polygons
            .par_iter()
            .map(|p| p.to_gds_impl(database_units))
            .collect();
        for b in polygon_bufs? {
            buffer.extend_from_slice(&b);
        }

        let text_bufs: Result<Vec<_>, _> = self
            .texts
            .par_iter()
            .map(|t| t.to_gds_impl(database_units))
            .collect();
        for b in text_bufs? {
            buffer.extend_from_slice(&b);
        }

        let ref_bufs: Result<Vec<_>, _> = self
            .references
            .par_iter()
            .map(|r| r.to_gds_impl(database_units))
            .collect();
        for b in ref_bufs? {
            buffer.extend_from_slice(&b);
        }

        let cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        write_u16_array_to_file(&mut buffer, &cell_tail)?;

        Ok(buffer)
    }
}
