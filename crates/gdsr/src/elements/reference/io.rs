use super::{Instance, Reference};
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::elements::Element;
use crate::error::GdsError;
use crate::traits::{Movable, ToGds, Transformable};
use crate::utils::io::{
    validate_col_row, write_element_tail_to_file, write_points_to_file,
    write_string_with_record_to_file, write_transformation_to_file, write_u16_array_to_file,
};

impl ToGds for Reference {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        match &self.instance {
            Instance::Cell(cell_name) => self.to_gds_impl_with_cell(database_units, cell_name),
            Instance::Element(element) => {
                self.to_gds_impl_with_element(database_units, element.as_ref().as_ref())
            }
        }
    }
}

impl Reference {
    fn to_gds_impl_with_element(
        &self,
        database_units: f64,
        element: &Element,
    ) -> Result<Vec<u8>, GdsError> {
        let grid = self.grid();
        let spacing_x = grid.spacing_x().unwrap_or_default();
        let spacing_y = grid.spacing_y().unwrap_or_default();

        let mut buf = Vec::new();
        for column_index in 0..grid.columns() {
            for row_index in 0..grid.rows() {
                let offset = (spacing_x * column_index) + (spacing_y * row_index);
                let rotated_offset =
                    offset.rotate_around_point(grid.angle(), &crate::Point::default());
                let final_position = grid.origin() + rotated_offset;

                let mut new_element = element.clone();
                if grid.x_reflection() {
                    new_element = new_element.reflect(0.0, grid.origin());
                }
                new_element = new_element.rotate(grid.angle(), grid.origin());
                new_element = new_element.scale(grid.magnification(), grid.origin());
                new_element = new_element.move_by(final_position - grid.origin());

                buf.extend_from_slice(&new_element.to_gds_impl(database_units)?);
            }
        }
        Ok(buf)
    }

    fn to_gds_impl_with_cell(
        &self,
        database_units: f64,
        cell_name: &str,
    ) -> Result<Vec<u8>, GdsError> {
        validate_col_row(self.grid().columns(), self.grid().rows())?;

        let is_single_instance = self.grid().columns() == 1 && self.grid().rows() == 1;

        let record = if is_single_instance {
            GDSRecord::SRef
        } else {
            GDSRecord::ARef
        };

        let mut buffer = Vec::new();

        let buffer_start = [4, combine_record_and_data_type(record, GDSDataType::NoData)];

        write_u16_array_to_file(&mut buffer, &buffer_start)?;

        write_string_with_record_to_file(&mut buffer, GDSRecord::SName, cell_name)?;

        let angle = self.grid().angle();
        let magnification = self.grid().magnification();
        let x_reflection = self.grid().x_reflection();

        write_transformation_to_file(&mut buffer, angle, magnification, x_reflection)?;

        if is_single_instance {
            let origin = self.grid().origin();
            let reference_points = [origin];
            write_points_to_file(&mut buffer, &reference_points, database_units)?;
        } else {
            let buffer_array = [
                8,
                combine_record_and_data_type(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger),
                self.grid().columns() as u16,
                self.grid().rows() as u16,
            ];

            write_u16_array_to_file(&mut buffer, &buffer_array)?;

            let origin = self
                .grid()
                .origin()
                .rotate_around_point(self.grid().angle(), &self.grid().origin());

            match (self.grid.spacing_x(), self.grid.spacing_y()) {
                (Some(spacing_x), Some(spacing_y)) => {
                    let point2 = ((origin + spacing_x) * self.grid().columns())
                        .rotate_around_point(self.grid().angle(), &origin);

                    let point3 = ((origin + spacing_y) * self.grid().rows())
                        .rotate_around_point(self.grid().angle(), &origin);

                    let reference_points = [origin, point2, point3];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
                (Some(spacing_x), None) => {
                    let point2 = ((origin + spacing_x) * self.grid().columns())
                        .rotate_around_point(self.grid().angle(), &origin);
                    let reference_points = [origin, point2, origin];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
                (None, Some(spacing_y)) => {
                    let point3 = ((origin + spacing_y) * self.grid().rows())
                        .rotate_around_point(self.grid().angle(), &origin);
                    let reference_points = [origin, origin, point3];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
                _ => {
                    let reference_points = [origin];
                    write_points_to_file(&mut buffer, &reference_points, database_units)?;
                }
            }
        }

        write_element_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }
}
