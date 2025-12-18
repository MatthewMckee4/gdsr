use std::{fs::File, io};

use super::{Instance, Reference};
use crate::{
    config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type},
    elements::Element,
    traits::ToGds,
    utils::io::{
        write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
        write_transformation_to_file, write_u16_array_to_file,
    },
};

impl ToGds for Reference {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()> {
        match &self.instance {
            Instance::Cell(cell_name) => self.to_gds_impl_with_cell(file, scale, cell_name),
            Instance::Element(element) => {
                self.to_gds_impl_with_element(file, scale, element.as_ref().as_ref())
            }
        }
    }
}

impl Reference {
    fn to_gds_impl_with_element(
        &self,
        file: &mut File,
        scale: f64,
        element: &Element,
    ) -> io::Result<()> {
        for element in self.get_elements_in_grid(element) {
            element.to_gds_impl(file, scale)?;
        }

        Ok(())
    }

    fn to_gds_impl_with_cell(
        &self,
        file: &mut File,
        scale: f64,
        cell_name: &str,
    ) -> io::Result<()> {
        let buffer_start = [
            4,
            combine_record_and_data_type(GDSRecord::ARef, GDSDataType::NoData),
        ];

        write_u16_array_to_file(file, &buffer_start)?;

        write_string_with_record_to_file(file, GDSRecord::SName, cell_name)?;

        write_transformation_to_file(
            file,
            self.grid().angle(),
            self.grid().magnification(),
            self.grid().x_reflection(),
        )?;

        let buffer_array = [
            8,
            combine_record_and_data_type(GDSRecord::ColRow, GDSDataType::TwoByteSignedInteger),
            self.grid().columns() as u16,
            self.grid().rows() as u16,
        ];

        write_u16_array_to_file(file, &buffer_array)?;

        let origin = self.grid().origin().to_integer_unit();
        let point2 = (self.grid().origin() + self.grid().spacing_x()) * self.grid().columns();
        let point3 = (self.grid().origin() + self.grid().spacing_y()) * self.grid().rows();

        let reference_points: Vec<_> = [origin, point2, point3]
            .iter()
            .map(|&p| p.rotate_around_point(self.grid().angle(), &origin))
            .collect();

        write_points_to_file(file, &reference_points, scale)?;

        write_element_tail_to_file(file)
    }
}
