use super::{Instance, Reference};
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::elements::Element;
use crate::traits::ToGds;
use crate::utils::io::{
    write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
    write_transformation_to_file, write_u16_array_to_file,
};

impl ToGds for Reference {
    fn to_gds_impl(
        &self,
        buffer: &mut impl std::io::Write,
        database_units: f64,
    ) -> std::io::Result<()> {
        match &self.instance {
            Instance::Cell(cell_name) => {
                self.to_gds_impl_with_cell(buffer, database_units, cell_name)
            }
            Instance::Element(element) => {
                self.to_gds_impl_with_element(buffer, database_units, element.as_ref().as_ref())
            }
        }
    }
}

impl Reference {
    fn to_gds_impl_with_element(
        &self,
        buffer: &mut impl std::io::Write,
        database_units: f64,
        element: &Element,
    ) -> std::io::Result<()> {
        for element in self.get_elements_in_grid(element) {
            element.to_gds_impl(buffer, database_units)?;
        }

        Ok(())
    }

    fn to_gds_impl_with_cell(
        &self,
        buffer: &mut impl std::io::Write,
        database_units: f64,
        cell_name: &str,
    ) -> std::io::Result<()> {
        let buffer_start = [
            4,
            combine_record_and_data_type(GDSRecord::ARef, GDSDataType::NoData),
        ];

        write_u16_array_to_file(buffer, &buffer_start)?;

        write_string_with_record_to_file(buffer, GDSRecord::SName, cell_name)?;

        let angle = self.grid().angle();
        let magnification = self.grid().magnification();
        let x_reflection = self.grid().x_reflection();

        write_transformation_to_file(buffer, angle, magnification, x_reflection)?;

        let buffer_array = [
            8,
            combine_record_and_data_type(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger),
            self.grid().columns() as u16,
            self.grid().rows() as u16,
        ];

        write_u16_array_to_file(buffer, &buffer_array)?;

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
                write_points_to_file(buffer, &reference_points, database_units)?;
            }
            (Some(spacing_x), None) => {
                let point2 = ((origin + spacing_x) * self.grid().columns())
                    .rotate_around_point(self.grid().angle(), &origin);
                let reference_points = [origin, point2, origin];
                write_points_to_file(buffer, &reference_points, database_units)?;
            }
            (None, Some(spacing_y)) => {
                let point3 = ((origin + spacing_y) * self.grid().rows())
                    .rotate_around_point(self.grid().angle(), &origin);
                let reference_points = [origin, origin, point3];
                write_points_to_file(buffer, &reference_points, database_units)?;
            }
            _ => {
                let reference_points = [origin];
                write_points_to_file(buffer, &reference_points, database_units)?;
            }
        }

        write_element_tail_to_file(buffer)
    }
}
