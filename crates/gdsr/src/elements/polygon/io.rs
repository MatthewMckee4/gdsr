use std::{fs::File, io};

use super::Polygon;
use crate::{
    config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type},
    traits::ToGds,
    utils::io::{write_element_tail_to_file, write_points_to_file, write_u16_array_to_file},
};

impl ToGds for Polygon {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()> {
        if self.points().len() > 8191 {
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

        write_u16_array_to_file(file, &polygon_head)?;

        write_points_to_file(file, self.points(), scale)?;

        write_element_tail_to_file(file)
    }
}
