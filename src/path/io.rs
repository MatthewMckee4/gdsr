use pyo3::{exceptions::PyValueError, prelude::*};
use std::fs::File;

use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    utils::io::{write_element_tail_to_file, write_points_to_file, write_u16_array_to_file},
};

use super::{path_type::PathType, Path};

impl Path {
    pub fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        if self.points.len() < 2 {
            return Err(PyValueError::new_err("Path must have at least 2 points"));
        }

        let path_type_value = self.path_type.unwrap_or(PathType::Square).value()? as u16;
        let width_value = (self.width.unwrap_or(0.0) * scale).round() as u32;

        let mut path_head = [
            4,
            combine_record_and_data_type(GDSRecord::Path, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer as u16,
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type as u16,
            6,
            combine_record_and_data_type(GDSRecord::PathType, GDSDataType::TwoByteSignedInteger),
            path_type_value,
            8,
            combine_record_and_data_type(GDSRecord::Width, GDSDataType::FourByteSignedInteger),
            (width_value >> 16) as u16,
            (width_value & 0xFFFF) as u16,
        ];

        file = write_u16_array_to_file(file, &mut path_head)?;

        file = write_points_to_file(file, &self.points, scale)?;

        file = write_element_tail_to_file(file)?;

        Ok(file)
    }
}
