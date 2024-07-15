use std::fs::File;
use std::io::Write;

use crate::{
    config::gds_file_types::{combine_record_and_data_type, GDSDataType, GDSRecord},
    utils::gds_format::write_u16_array_to_file,
};
use log::warn;
use pyo3::prelude::*;

use super::Polygon;

impl Polygon {
    pub fn _to_gds(&self, mut file: File, scale: f64) -> PyResult<File> {
        if self.points.len() > 8190 {
            warn!(
                "{} has more than 8190 points, This may cause errors in the future.",
                self
            );
        }

        let mut polygon_head = [
            4,
            combine_record_and_data_type(GDSRecord::Boundary, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer as u16,
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type as u16,
        ];

        write_u16_array_to_file(&mut polygon_head, &mut file)?;

        let xy_head =
            combine_record_and_data_type(GDSRecord::XY, GDSDataType::FourByteSignedInteger);
        let points_length = self.points.len();
        let mut i0 = 0;

        let mut xy_header_buffer = vec![0u8; 4];
        let mut points_buffer = Vec::with_capacity(8 * 8190);

        while i0 < points_length {
            let i1 = (i0 + 8190).min(points_length);
            let record_length = 4 + 8 * (i1 - i0);

            xy_header_buffer[0..2].copy_from_slice(&(record_length as u16).to_be_bytes());
            xy_header_buffer[2..4].copy_from_slice(&xy_head.to_be_bytes());

            file.write_all(&xy_header_buffer)?;

            points_buffer.clear();
            for point in &self.points[i0..i1] {
                let scaled_x = (point.x * scale).round() as i32;
                let scaled_y = (point.y * scale).round() as i32;

                points_buffer.extend_from_slice(&scaled_x.to_be_bytes());
                points_buffer.extend_from_slice(&scaled_y.to_be_bytes());
            }

            file.write_all(&points_buffer)?;

            i0 = i1;
        }

        let mut polygon_tail = [
            4,
            combine_record_and_data_type(GDSRecord::EndEl, GDSDataType::NoData),
        ];

        write_u16_array_to_file(&mut polygon_tail, &mut file)?;

        Ok(file)
    }
}
