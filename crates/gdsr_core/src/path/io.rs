use std::{fs::File, io::{self, Write}};

use crate::{
    Point,
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    traits::ToGds,
};

use super::{path_type::PathType, Path};

fn write_u16_array_to_file(file: &mut File, array: &[u16]) -> io::Result<()> {
    for &value in array {
        file.write_all(&value.to_be_bytes())?;
    }
    Ok(())
}

fn write_points_to_file(file: &mut File, points: &[Point], scale: f64) -> io::Result<()> {
    let record_size = 4 + (points.len() * 8) as u16;
    let xy_header_buffer = [
        record_size,
        combine_record_and_data_type(GDSRecord::XY, GDSDataType::FourByteSignedInteger),
    ];

    write_u16_array_to_file(file, &xy_header_buffer)?;

    for point in points {
        let scaled_x = (point.x() * scale).round() as i32;
        let scaled_y = (point.y() * scale).round() as i32;

        file.write_all(&scaled_x.to_be_bytes())?;
        file.write_all(&scaled_y.to_be_bytes())?;
    }
    
    Ok(())
}

fn write_element_tail_to_file(file: &mut File) -> io::Result<()> {
    let tail = [4, combine_record_and_data_type(GDSRecord::EndEl, GDSDataType::NoData)];
    write_u16_array_to_file(file, &tail)
}

impl ToGds for Path {
    fn _to_gds(&self, file: &mut File, scale: f64) -> io::Result<()> {
        if self.points.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path must have at least 2 points"
            ));
        }

        let mut path_head = [
            4,
            combine_record_and_data_type(GDSRecord::Path, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer as u16,
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type as u16,
        ];

        write_u16_array_to_file(file, &mut path_head)?;

        if self.path_type.is_some() {
            let path_type_value = self.path_type.unwrap_or(PathType::Square).value() as u16;

            let mut path_type_head = [
                6,
                combine_record_and_data_type(
                    GDSRecord::PathType,
                    GDSDataType::TwoByteSignedInteger,
                ),
                path_type_value,
            ];

            write_u16_array_to_file(file, &mut path_type_head)?;
        }

        if self.width.is_some() {
            let width_value = (self.width.unwrap_or(0.0) * scale).round() as u32;
            let mut width_head = [
                8,
                combine_record_and_data_type(GDSRecord::Width, GDSDataType::FourByteSignedInteger),
            ];

            write_u16_array_to_file(file, &mut width_head)?;

            let bytes = width_value.to_be_bytes();

            file.write_all(&bytes)?;
        }

        write_points_to_file(file, &self.points, scale)?;

        write_element_tail_to_file(file)?;

        Ok(())
    }
}
