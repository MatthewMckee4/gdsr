use log::warn;
use pyo3::prelude::*;
use std::fs::File;

use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    utils::io::{write_element_tail_to_file, write_points_to_file, write_u16_array_to_file},
};

use super::Polygon;

impl Polygon {
    pub fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        if self.points.len() > 8190 {
            warn!(
                "{} has more than 8190 points, This may cause errors in the future.",
                self
            );
        }

        let mut polygon_head = [
            4,
            combine_record_and_data_type(GDSRecord::Boundary, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer as u16,
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type as u16,
        ];

        file = write_u16_array_to_file(file, &mut polygon_head)?;

        file = write_points_to_file(file, &self.points, scale)?;

        file = write_element_tail_to_file(file)?;

        Ok(file)
    }
}
