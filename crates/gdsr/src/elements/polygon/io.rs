use std::io;

use super::Polygon;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::traits::ToGds;
use crate::utils::io::{
    MAX_POINTS, write_element_tail_to_file, write_points_to_file, write_u16_array_to_file,
};

impl ToGds for Polygon {
    fn to_gds_impl(&self, buffer: &mut impl std::io::Write, database_units: f64) -> io::Result<()> {
        if self.points().len() > MAX_POINTS {
            return Ok(());
        }

        let polygon_head = [
            4,
            combine_record_and_data_type(GDSRecord::Boundary, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer(),
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type(),
        ];

        write_u16_array_to_file(buffer, &polygon_head)?;

        write_points_to_file(buffer, self.points(), database_units)?;

        write_element_tail_to_file(buffer)
    }
}
