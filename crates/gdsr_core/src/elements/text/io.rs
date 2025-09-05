use std::{fs::File, io};

use super::{Text, utils::get_presentation_value};
use crate::{
    CoordNum,
    config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type},
    traits::ToGds,
    utils::io::{
        write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
        write_transformation_to_file, write_u16_array_to_file,
    },
};

impl<DatabaseUnitT: CoordNum> ToGds for Text<DatabaseUnitT> {
    fn to_gds_impl(&self, file: &mut File, scale: f64) -> io::Result<()> {
        let buffer_start = vec![
            4,
            combine_record_and_data_type(GDSRecord::Text, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer(),
            6,
            combine_record_and_data_type(GDSRecord::TextType, GDSDataType::TwoByteSignedInteger),
            0,
            6,
            combine_record_and_data_type(GDSRecord::Presentation, GDSDataType::BitArray),
            get_presentation_value(
                *self.vertical_presentation(),
                *self.horizontal_presentation(),
            ),
        ];

        write_u16_array_to_file(file, &buffer_start)?;

        write_transformation_to_file(
            file,
            self.angle(),
            self.magnification(),
            self.x_reflection(),
        )?;

        write_points_to_file(file, &[*self.origin()], scale, &|val| val.to_integer())?;

        write_string_with_record_to_file(file, GDSRecord::String, self.text())?;

        write_element_tail_to_file(file)
    }
}
