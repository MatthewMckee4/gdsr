use pyo3::{exceptions::PyValueError, prelude::*};
use std::fs::File;

use crate::{
    config::gds_file_types::{
        combine_record_and_data_type, GDSDataType, GDSRecord, MAX_POLYGON_POINTS,
    },
    traits::ToGds,
    utils::io::{write_element_tail_to_file, write_points_to_file, write_u16_array_to_file},
};

use super::Polygon;

impl ToGds for Polygon {
    fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        if self.points.len() > MAX_POLYGON_POINTS {
            Err(PyValueError::new_err(format!(
                "A polygon can only have a maximum of {} points",
                MAX_POLYGON_POINTS
            )))?;
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
