use std::io::Write;

use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_data_type, validate_layer, write_element_tail_to_file, write_points_to_file,
    write_u16_array_to_file,
};
use crate::{DataType, Dimensions, Layer, Movable, Point, Transformable, Unit};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PathType {
    #[default]
    Square = 0,
    Round = 1,
    Overlap = 2,
}

impl PathType {
    pub const fn new(value: i32) -> Self {
        match value {
            1 => Self::Round,
            2 => Self::Overlap,
            _ => Self::Square,
        }
    }

    pub const fn value(&self) -> u16 {
        *self as u16
    }

    pub fn values() -> Vec<Self> {
        vec![Self::Square, Self::Round, Self::Overlap]
    }
}

/// An open path defined by a sequence of points, with optional width and end cap type.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Path {
    pub(crate) points: Vec<Point>,
    pub(crate) layer: Layer,
    pub(crate) data_type: DataType,
    pub(crate) r#type: Option<PathType>,
    pub(crate) width: Option<Unit>,
}

impl Path {
    /// Creates a new path from the given points, layer, data type, optional end cap type, and optional width.
    pub fn new(
        points: impl IntoIterator<Item = Point>,
        layer: Layer,
        data_type: DataType,
        path_type: Option<PathType>,
        width: Option<Unit>,
    ) -> Self {
        Self {
            points: points.into_iter().collect(),
            layer,
            data_type,
            r#type: path_type,
            width,
        }
    }

    /// Returns the path's points.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Returns the layer number.
    pub const fn layer(&self) -> Layer {
        self.layer
    }

    /// Returns the data type.
    pub const fn data_type(&self) -> DataType {
        self.data_type
    }

    /// Returns the end cap type, if set.
    pub const fn path_type(&self) -> &Option<PathType> {
        &self.r#type
    }

    /// Returns the path width, if set.
    pub const fn width(&self) -> Option<Unit> {
        self.width
    }

    /// Converts all points and width to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            points: self.points.iter().map(Point::to_integer_unit).collect(),
            width: self.width.map(Unit::to_integer_unit),
            ..self
        }
    }

    /// Converts all points and width to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            points: self.points.iter().map(Point::to_float_unit).collect(),
            width: self.width.map(Unit::to_float_unit),
            ..self
        }
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Path with {} points on layer {} with data type {}, {:?} and width {}",
            self.points().len(),
            self.layer(),
            self.data_type(),
            self.path_type().unwrap_or_default(),
            self.width().unwrap_or_default()
        )
    }
}

impl Transformable for Path {
    fn transform_impl(mut self, transformation: &crate::Transformation) -> Self {
        self.points = self
            .points()
            .iter()
            .map(|point| point.transform(transformation))
            .collect();

        self
    }
}

impl Movable for Path {
    fn move_to(self, target: Point) -> Self {
        let Some(first_point) = self.points().first() else {
            return self;
        };
        let delta = target - *first_point;
        self.move_by(delta)
    }
}

impl Dimensions for Path {
    fn bounding_box(&self) -> (Point, Point) {
        crate::geometry::bounding_box(&self.points)
    }
}

impl ToGds for Path {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        validate_layer(self.layer())?;
        validate_data_type(self.data_type())?;

        if self.points().len() < 2 {
            return Err(GdsError::ValidationError {
                message: "Path must have at least 2 points".to_string(),
            });
        }

        let mut buffer = Vec::new();

        let path_head = [
            4,
            combine_record_and_data_type(GDSRecord::Path, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer().value(),
            6,
            combine_record_and_data_type(GDSRecord::DataType, GDSDataType::TwoByteSignedInteger),
            self.data_type().value(),
        ];

        write_u16_array_to_file(&mut buffer, &path_head)?;

        if let Some(path_type) = self.path_type() {
            let path_type_value = path_type.value();

            let path_type_head = [
                6,
                combine_record_and_data_type(
                    GDSRecord::PathType,
                    GDSDataType::TwoByteSignedInteger,
                ),
                path_type_value,
            ];

            write_u16_array_to_file(&mut buffer, &path_type_head)?;
        }

        if let Some(width) = self.width() {
            let scaled_width = width.scale_to(database_units);
            let width_value = scaled_width.as_integer_unit().value as u32;

            let width_head = [
                8,
                combine_record_and_data_type(GDSRecord::Width, GDSDataType::FourByteSignedInteger),
            ];

            write_u16_array_to_file(&mut buffer, &width_head)?;

            let bytes = width_value.to_be_bytes();

            buffer.write_all(&bytes)?;
        }

        write_points_to_file(&mut buffer, self.points(), database_units)?;

        write_element_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_type_new() {
        assert_eq!(PathType::new(0), PathType::Square);
        assert_eq!(PathType::new(1), PathType::Round);
        assert_eq!(PathType::new(2), PathType::Overlap);
        assert_eq!(PathType::new(-1), PathType::Square);
        assert_eq!(PathType::new(999), PathType::Square);
    }

    #[test]
    fn test_path_type_value() {
        assert_eq!(PathType::Square.value(), 0);
        assert_eq!(PathType::Round.value(), 1);
        assert_eq!(PathType::Overlap.value(), 2);
    }

    #[test]
    fn test_path_type_values() {
        let values = PathType::values();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&PathType::Square));
        assert!(values.contains(&PathType::Round));
        assert!(values.contains(&PathType::Overlap));
    }

    #[test]
    fn test_path_type_default() {
        assert_eq!(PathType::default(), PathType::Square);
    }

    #[test]
    fn test_path_type_debug() {
        insta::assert_snapshot!(format!("{:?}", PathType::Square), @"Square");
        insta::assert_snapshot!(format!("{:?}", PathType::Round), @"Round");
        insta::assert_snapshot!(format!("{:?}", PathType::Overlap), @"Overlap");
    }

    #[test]
    fn test_path_type_clone_and_copy() {
        let path_type = PathType::Round;
        let cloned = path_type;
        let copied = path_type;

        assert_eq!(path_type, cloned);
        assert_eq!(path_type, copied);
    }

    #[test]
    fn test_path_type_partial_eq() {
        assert_eq!(PathType::Square, PathType::Square);
        assert_ne!(PathType::Square, PathType::Round);
        assert_ne!(PathType::Round, PathType::Overlap);
    }

    #[test]
    fn test_path_creation() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(100, 100, 1e-9)];
        let path = Path::new(
            points.clone(),
            Layer::new(1),
            DataType::new(2),
            Some(PathType::Round),
            Some(Unit::default_integer(10)),
        );

        assert_eq!(path.points(), &points);
        assert_eq!(path.layer(), Layer::new(1));
        assert_eq!(path.data_type(), DataType::new(2));
        assert_eq!(path.path_type(), &Some(PathType::Round));
        assert_eq!(path.width(), Some(Unit::default_integer(10)));
    }

    #[test]
    fn test_path_default() {
        let path = Path::default();

        assert!(path.points().is_empty());
        assert_eq!(path.layer(), Layer::new(0));
        assert_eq!(path.data_type(), DataType::new(0));
        assert_eq!(path.path_type(), &None);
        assert_eq!(path.width(), None);
    }

    #[test]
    fn test_path_display() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(100, 100, 1e-9)];
        let path = Path::new(
            points,
            Layer::new(5),
            DataType::new(10),
            Some(PathType::Square),
            Some(Unit::default_integer(20)),
        );

        insta::assert_snapshot!(path.to_string(), @"Path with 2 points on layer 5 with data type 10, Square and width 20 (1.000e-9)");
    }

    #[test]
    fn test_path_clone_and_partial_eq() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)];
        let path1 = Path::new(
            points.clone(),
            Layer::new(1),
            DataType::new(2),
            Some(PathType::Round),
            Some(Unit::default_integer(5)),
        );
        let path2 = path1.clone();

        assert_eq!(path1, path2);

        let path3 = Path::new(
            points,
            Layer::new(1),
            DataType::new(2),
            Some(PathType::Square),
            Some(Unit::default_integer(5)),
        );
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_path_to_integer_unit() {
        let points = vec![Point::float(1.5, 2.5, 1e-6), Point::float(10.0, 10.0, 1e-6)];
        let path = Path::new(
            points,
            Layer::new(1),
            DataType::new(0),
            Some(PathType::Round),
            Some(Unit::default_float(5.0)),
        );
        let converted = path.to_integer_unit();

        for point in converted.points() {
            assert_eq!(*point, point.to_integer_unit());
        }
        assert_eq!(
            converted.width(),
            Some(Unit::default_float(5.0).to_integer_unit())
        );
    }

    #[test]
    fn test_path_to_float_unit() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(100, 100, 1e-9)];
        let path = Path::new(
            points,
            Layer::new(1),
            DataType::new(0),
            None,
            Some(Unit::default_integer(10)),
        );
        let converted = path.to_float_unit();

        for point in converted.points() {
            assert_eq!(*point, point.to_float_unit());
        }
        assert_eq!(
            converted.width(),
            Some(Unit::default_integer(10).to_float_unit())
        );
    }

    #[test]
    fn test_path_to_integer_unit_no_width() {
        let points = vec![Point::float(1.0, 2.0, 1e-6)];
        let path = Path::new(points, Layer::new(0), DataType::new(0), None, None);
        let converted = path.to_integer_unit();

        assert_eq!(converted.width(), None);
    }

    #[test]
    fn test_path_with_different_unit_points() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::float(100.0, 100.0, 1e-6)];
        let path = Path::new(points, Layer::new(0), DataType::new(0), None, None);
        assert_eq!(path.points().len(), 2);
    }

    #[test]
    fn test_path_bounding_box() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 5, 1e-9),
            Point::integer(20, -3, 1e-9),
        ];
        let path = Path::new(points, Layer::new(1), DataType::new(0), None, None);
        let (min, max) = path.bounding_box();
        assert_eq!(min, Point::integer(0, -3, 1e-9));
        assert_eq!(max, Point::integer(20, 5, 1e-9));
    }

    #[test]
    fn test_path_bounding_box_empty() {
        let path = Path::default();
        let (min, max) = path.bounding_box();
        assert_eq!(min, Point::default());
        assert_eq!(max, Point::default());
    }

    #[test]
    fn test_path_bounding_box_single_point() {
        let points = vec![Point::integer(5, 10, 1e-9)];
        let path = Path::new(points, Layer::new(1), DataType::new(0), None, None);
        let (min, max) = path.bounding_box();
        assert_eq!(min, Point::integer(5, 10, 1e-9));
        assert_eq!(max, Point::integer(5, 10, 1e-9));
    }
}
