use chrono::{Datelike, Local, Timelike};

use crate::Cell;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_structure_name, write_string_with_record_to_file, write_u16_array_to_file,
};

impl ToGds for Cell {
    fn to_gds_impl(
        &self,
        buffer: &mut impl std::io::Write,
        database_units: f64,
    ) -> Result<(), GdsError> {
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

        write_u16_array_to_file(buffer, &cell_head)?;

        write_string_with_record_to_file(buffer, GDSRecord::StrName, &self.name)?;

        for path in &self.paths {
            path.to_gds_impl(buffer, database_units)?;
        }

        for polygon in &self.polygons {
            polygon.to_gds_impl(buffer, database_units)?;
        }

        for text in &self.texts {
            text.to_gds_impl(buffer, database_units)?;
        }

        for reference in &self.references {
            reference.to_gds_impl(buffer, database_units)?;
        }

        let cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        write_u16_array_to_file(buffer, &cell_tail)?;

        Ok(())
    }
}
