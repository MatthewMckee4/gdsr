use std::{fs::File, io};

use chrono::{Datelike, Local, Timelike};

use crate::{
    Cell, CoordNum,
    config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type},
    traits::ToGds,
    utils::io::{write_string_with_record_to_file, write_u16_array_to_file},
};

impl<DatabaseUnitT: CoordNum> ToGds for Cell<DatabaseUnitT> {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()> {
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

        write_u16_array_to_file(file, &cell_head)?;

        write_string_with_record_to_file(file, GDSRecord::StrName, &self.name)?;

        for path in &self.paths {
            path.to_gds_impl(file, scale)?;
        }

        for polygon in &self.polygons {
            polygon.to_gds_impl(file, scale)?;
        }

        for text in &self.texts {
            text.to_gds_impl(file, scale)?;
        }

        for reference in &self.references {
            reference.to_gds_impl(file, scale)?;
        }

        let cell_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndStr, GDSDataType::NoData),
        ];

        write_u16_array_to_file(file, &cell_tail)?;

        Ok(())
    }
}
