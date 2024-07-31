use pyo3::prelude::*;
use std::fs::File;

use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    element::Element,
    traits::{Movable, Rotatable, Scalable, ToGds},
    utils::io::{
        write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
        write_transformation_to_file, write_u16_array_to_file,
    },
};

use super::{Reference, ReferenceInstance};

impl ToGds for Reference {
    fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        match &self.instance {
            ReferenceInstance::Cell(cell) => {
                file = self._to_gds_with_cell(file, scale, &cell.name)?
            }
            ReferenceInstance::Element(element) => {
                file = self._to_gds_with_element(file, scale, &element)?;
            }
        }

        Ok(file)
    }
}

impl Reference {
    fn _to_gds_with_element(
        &self,
        mut file: File,
        scale: f64,
        element: &Element,
    ) -> PyResult<File> {
        for column_index in 0..self.grid.columns {
            for row_index in 0..self.grid.rows {
                let origin = self.grid.origin
                    + self.grid.spacing_x * column_index as f64
                    + self.grid.spacing_y * row_index as f64;

                let new_element = element
                    .copy()
                    .scale(
                        if self.grid.x_reflection { -1.0 } else { 1.0 },
                        self.grid.origin,
                    )
                    .scale(self.grid.magnification, self.grid.origin)
                    .rotate(self.grid.angle, self.grid.origin)
                    .move_by(origin.rotate(self.grid.angle, self.grid.origin).scale(
                        if self.grid.x_reflection { -1.0 } else { 1.0 },
                        self.grid.origin,
                    ))
                    .copy();

                file = new_element._to_gds(file, scale)?;
            }
        }

        Ok(file)
    }

    fn _to_gds_with_cell(&self, mut file: File, scale: f64, cell_name: &str) -> PyResult<File> {
        let mut buffer_start = [
            4,
            combine_record_and_data_type(GDSRecord::ARef, GDSDataType::NoData),
        ];

        file = write_u16_array_to_file(file, &mut buffer_start)?;

        file = write_string_with_record_to_file(file, GDSRecord::SName, cell_name)?;

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

        let origin = self.grid.origin;
        let point2 = self.grid.origin + self.grid.spacing_x * self.grid.columns as f64;
        let point3 = self.grid.origin + self.grid.spacing_y * self.grid.rows as f64;

        let mut points = vec![origin, point2, point3];

        points = points
            .iter()
            .map(|&p| p.rotate(self.grid.angle, origin))
            .collect();

        file = write_points_to_file(file, &points, scale)?;

        file = write_element_tail_to_file(file)?;

        Ok(file)
    }
}
