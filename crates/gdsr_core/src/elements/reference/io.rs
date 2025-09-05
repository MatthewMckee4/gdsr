use crate::{
    CoordNum,
    config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type},
    elements::Element,
    traits::ToGds,
    utils::{
        general::point_to_database_float,
        io::{
            write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
            write_transformation_to_file, write_u16_array_to_file,
        },
    },
};
use geo::algorithm::rotate::Rotate;
use std::{fs::File, io};

use super::{Instance, Reference};

impl<DatabaseUnitT: CoordNum> ToGds for Reference<DatabaseUnitT> {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()> {
        match &self.instance {
            Instance::Cell(cell) => self.to_gds_impl_with_cell(file, scale, &cell.name),
            Instance::Element(element) => {
                self.to_gds_impl_with_element(file, scale, element.as_ref().as_ref())
            }
        }
    }
}

impl<DatabaseUnitT: CoordNum> Reference<DatabaseUnitT> {
    fn to_gds_impl_with_element(
        &self,
        file: &mut File,
        scale: f64,
        element: &Element<DatabaseUnitT>,
    ) -> io::Result<()> {
        for element in self._get_elements_in_grid(element) {
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
        let mut buffer_start = [
            4,
            combine_record_and_data_type(GDSRecord::ARef, GDSDataType::NoData),
        ];

        write_u16_array_to_file(file, &mut buffer_start)?;

        write_string_with_record_to_file(file, GDSRecord::SName, cell_name)?;

        write_transformation_to_file(
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

        write_u16_array_to_file(file, &mut buffer_array)?;

        let origin = point_to_database_float(self.grid.origin);
        let point2 = point_to_database_float(self.grid.origin + self.grid.spacing_x)
            * (self.grid.columns as f64);
        let point3 = point_to_database_float(self.grid.origin + self.grid.spacing_y)
            * (self.grid.rows as f64);

        let reference_points: Vec<_> = vec![origin, point2, point3]
            .iter()
            .map(|&p| p.rotate_around_point(self.grid.angle.into(), origin))
            .collect();

        write_points_to_file(file, &reference_points, scale, &|val| val.to_integer())?;

        write_element_tail_to_file(file)
    }
}
