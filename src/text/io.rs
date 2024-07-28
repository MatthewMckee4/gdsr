use pyo3::prelude::*;
use std::fs::File;

use crate::utils::io::write_string_with_record_to_file;
use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    utils::io::{
        write_eight_byte_real_to_file, write_element_tail_to_file, write_points_to_file,
        write_u16_array_to_file,
    },
};

use super::utils::get_presentation_value;
use super::Text;

impl Text {
    pub fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        let mut buffer_start = vec![
            4,
            combine_record_and_data_type(GDSRecord::Text, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer as u16,
            6,
            combine_record_and_data_type(GDSRecord::TextType, GDSDataType::TwoByteSignedInteger),
            0,
            6,
            combine_record_and_data_type(GDSRecord::Presentation, GDSDataType::BitArray),
            get_presentation_value(self.vertical_presentation, self.horizontal_presentation)?,
        ];

        file = write_u16_array_to_file(file, &mut buffer_start)?;

        let transform_applied = self.angle != 0.0 || self.magnification != 1.0 || self.x_reflection;
        if transform_applied {
            let mut buffer_flags = [
                6,
                combine_record_and_data_type(GDSRecord::STrans, GDSDataType::BitArray),
                if self.x_reflection { 0x8000 } else { 0x0000 },
            ];

            file = write_u16_array_to_file(file, &mut buffer_flags)?;

            if self.magnification != 1.0 {
                let mut buffer_mag = [
                    12,
                    combine_record_and_data_type(GDSRecord::Mag, GDSDataType::EightByteReal),
                ];
                file = write_u16_array_to_file(file, &mut buffer_mag)?;
                file = write_eight_byte_real_to_file(file, self.magnification)?;
            }

            if self.angle != 0.0 {
                let mut buffer_rot = [
                    12,
                    combine_record_and_data_type(GDSRecord::Angle, GDSDataType::EightByteReal),
                ];
                file = write_u16_array_to_file(file, &mut buffer_rot)?;
                file = write_eight_byte_real_to_file(file, self.angle)?;
            }
        }

        file = write_points_to_file(file, &[self.origin], scale)?;

        file = write_string_with_record_to_file(file, GDSRecord::String, &self.text)?;

        file = write_element_tail_to_file(file)?;

        Ok(file)
    }
}
