use pyo3::prelude::*;
use std::fs::File;

use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    element::Element,
    point::Point,
    traits::{Movable, Rotatable, Scalable, ToGds},
    utils::io::{
        write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
        write_transformation_to_file, write_u16_array_to_file,
    },
};

use super::{Instance, Reference};

impl ToGds for Reference {
    fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        Python::with_gil(|py| {
            match &self.instance {
                Instance::Cell(cell) => {
                    file = self._to_gds_with_cell(file, scale, &cell.borrow(py).name)?;
                }
                Instance::Element(element) => {
                    file = self._to_gds_with_element(file, scale, element)?;
                }
            }
            Ok(file)
        })
    }
}

impl Reference {
    fn _to_gds_with_element(
        &self,
        mut file: File,
        scale: f64,
        element: &Element,
    ) -> PyResult<File> {
        let grid = Python::with_gil(|py| self.grid.borrow(py).clone());
        for column_index in 0..grid.columns {
            for row_index in 0..grid.rows {
                let origin = grid.origin
                    + grid.spacing_x * column_index as f64
                    + grid.spacing_y * row_index as f64;

                let new_element = element
                    .copy()?
                    .scale(if grid.x_reflection { -1.0 } else { 1.0 }, grid.origin)
                    .scale(grid.magnification, grid.origin)
                    .rotate(grid.angle, Point::default())
                    .move_by(
                        origin
                            .rotate(grid.angle, grid.origin)
                            .scale(if grid.x_reflection { -1.0 } else { 1.0 }, grid.origin),
                    )
                    .copy()?;

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

        let grid = Python::with_gil(|py| self.grid.borrow(py).clone());

        file =
            write_transformation_to_file(file, grid.angle, grid.magnification, grid.x_reflection)?;

        let mut buffer_array = [
            8,
            combine_record_and_data_type(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger),
            grid.columns as u16,
            grid.rows as u16,
        ];

        file = write_u16_array_to_file(file, &mut buffer_array)?;

        let origin = grid.origin;
        let point2 = grid.origin + grid.spacing_x * grid.columns as f64;
        let point3 = grid.origin + grid.spacing_y * grid.rows as f64;

        let mut points = vec![origin, point2, point3];

        points = points
            .iter()
            .map(|&p| p.rotate(grid.angle, origin))
            .collect();

        file = write_points_to_file(file, &points, scale)?;

        file = write_element_tail_to_file(file)?;

        Ok(file)
    }
}
