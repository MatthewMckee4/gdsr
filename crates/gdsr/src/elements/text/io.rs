use super::Text;
use super::utils::get_presentation_value;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::traits::ToGds;
use crate::utils::io::{
    write_element_tail_to_file, write_points_to_file, write_string_with_record_to_file,
    write_transformation_to_file, write_u16_array_to_file,
};

impl ToGds for Text {
    fn to_gds_impl(
        &self,
        buffer: &mut impl std::io::Write,
        database_units: f64,
    ) -> std::io::Result<()> {
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

        write_u16_array_to_file(buffer, &buffer_start)?;

        let angle = self.angle();
        let magnification = self.magnification();
        let x_reflection = self.x_reflection();

        write_transformation_to_file(buffer, angle, magnification, x_reflection)?;

        write_points_to_file(buffer, &[*self.origin()], database_units)?;

        write_string_with_record_to_file(buffer, GDSRecord::String, self.text())?;

        write_element_tail_to_file(buffer)
    }
}
