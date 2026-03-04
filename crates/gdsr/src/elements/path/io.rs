use std::io::Write;

use super::Path;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_data_type, validate_layer, write_element_tail_to_file, write_points_to_file,
    write_u16_array_to_file,
};

impl ToGds for Path {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        validate_layer(self.layer())?;
        validate_data_type(self.data_type())?;

        if self.points().len() < 2 {
            return Err(GdsError::ValidationError {
                message: "Path must have at least 2 points".to_string(),
            });
        }

        let mut buffer = Vec::new();

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

        write_u16_array_to_file(&mut buffer, &path_head)?;

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

            write_u16_array_to_file(&mut buffer, &path_type_head)?;
        }

        if let Some(width) = self.width() {
            let scaled_width = width.scale_to(database_units);
            let width_value = scaled_width.as_integer_unit().value as u32;

            let width_head = [
                8,
                combine_record_and_data_type(GDSRecord::Width, GDSDataType::FourByteSignedInteger),
            ];

            write_u16_array_to_file(&mut buffer, &width_head)?;

            let bytes = width_value.to_be_bytes();

            buffer.write_all(&bytes)?;
        }

        write_points_to_file(&mut buffer, self.points(), database_units)?;

        write_element_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }
}
