use super::Polygon;
use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    MAX_POINTS, MIN_POLYGON_POINTS, validate_data_type, validate_layer, write_element_tail_to_file,
    write_points_to_file, write_u16_array_to_file,
};

impl ToGds for Polygon {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        if self.points().len() > MAX_POINTS {
            return Err(GdsError::ValidationError {
                message: format!(
                    "Polygon has {} points, which exceeds the maximum of {}",
                    self.points().len(),
                    MAX_POINTS
                ),
            });
        }

        validate_layer(self.layer())?;
        validate_data_type(self.data_type())?;

        if self.points().len() < MIN_POLYGON_POINTS {
            return Err(GdsError::ValidationError {
                message: format!(
                    "Polygon must have at least {MIN_POLYGON_POINTS} points (3 vertices + closing point), got {}",
                    self.points().len()
                ),
            });
        }

        let mut buffer = Vec::new();

        let polygon_head = [
            4,
            combine_record_and_data_type(GDSRecord::Boundary, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer(),
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type(),
        ];

        write_u16_array_to_file(&mut buffer, &polygon_head)?;

        write_points_to_file(&mut buffer, self.points(), database_units)?;

        write_element_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }
}
