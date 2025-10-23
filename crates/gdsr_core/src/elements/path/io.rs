use std::{
    fs::File,
    io::{self, Write},
};

use super::Path;
use crate::{
    config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type},
    traits::ToGds,
    utils::io::{write_element_tail_to_file, write_points_to_file, write_u16_array_to_file},
};

impl ToGds for Path {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()> {
        if self.points().len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path must have at least 2 points",
            ));
        }

        let path_head = [
            4,
            combine_record_and_data_type(GDSRecord::Path, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer(),
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type(),
        ];

        write_u16_array_to_file(file, &path_head)?;

        if let Some(path_type) = self.path_type() {
            let path_type_value = path_type.value();

            let path_type_head = [
                6,
                combine_record_and_data_type(
                    GDSRecord::PathType,
                    GDSDataType::TwoByteSignedInteger,
                ),
                path_type_value,
            ];

            write_u16_array_to_file(file, &path_type_head)?;
        }

        if let Some(width) = self.width() {
            let width_value = (width * scale).round() as u32;

            let width_head = [
                8,
                combine_record_and_data_type(GDSRecord::Width, GDSDataType::FourByteSignedInteger),
            ];

            write_u16_array_to_file(file, &width_head)?;

            let bytes = width_value.to_be_bytes();

            file.write_all(&bytes)?;
        }

        write_points_to_file(file, self.points(), scale)?;

        write_element_tail_to_file(file)
    }
}
