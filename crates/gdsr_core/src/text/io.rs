use pyo3::prelude::*;
use std::fs::File;

use crate::traits::ToGds;
use crate::utils::io::{write_string_with_record_to_file, write_transformation_to_file};
use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    utils::io::{write_element_tail_to_file, write_points_to_file, write_u16_array_to_file},
};

use super::utils::get_presentation_value;
use super::Text;

impl ToGds for Text {
    fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
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

        file =
            write_transformation_to_file(file, self.angle, self.magnification, self.x_reflection)?;

        file = write_points_to_file(file, &[self.origin], scale)?;

        file = write_string_with_record_to_file(file, GDSRecord::String, &self.text)?;

        file = write_element_tail_to_file(file)?;

        Ok(file)
    }
}
