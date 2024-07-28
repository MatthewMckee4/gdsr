use pyo3::prelude::*;
use std::fs::File;

use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    traits::ToGds,
    utils::io::{
        write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
        write_transformation_to_file, write_u16_array_to_file,
    },
};

use super::CellReference;

impl ToGds for CellReference {
    fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        let mut buffer_start = [
            4,
            combine_record_and_data_type(GDSRecord::ARef, GDSDataType::NoData),
        ];

        file = write_u16_array_to_file(file, &mut buffer_start)?;

        file = write_string_with_record_to_file(file, GDSRecord::SName, &self.cell.name)?;

        file = write_transformation_to_file(
            file,
            self.grid.angle,
            self.grid.magnification,
            self.grid.x_reflection,
        )?;

        let mut buffer_array = [
            8,
            combine_record_and_data_type(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger),
            self.grid.columns as u16,
            self.grid.rows as u16,
        ];

        file = write_u16_array_to_file(file, &mut buffer_array)?;

        file = write_points_to_file(
            file,
            &[
                self.grid.origin,
                self.grid.origin + self.grid.spacing_x * self.grid.columns as f64,
                self.grid.origin + self.grid.spacing_y * self.grid.rows as f64,
            ],
            scale,
        )?;

        file = write_element_tail_to_file(file)?;

        Ok(file)
    }
}
