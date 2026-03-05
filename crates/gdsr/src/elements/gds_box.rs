use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_data_type, validate_layer, write_element_tail_to_file, write_points_to_file,
    write_u16_array_to_file,
};
use crate::{DataType, Dimensions, Layer, LayerMapping, Movable, Point, Transformable};

/// A GDS II Box element defined by two diagonal corners on a specific layer.
///
/// Internally stores the bottom-left and top-right corners. When written to GDS,
/// the 5 closed points (4 corners + closing) are generated automatically.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GdsBox {
    pub(crate) bottom_left: Point,
    pub(crate) top_right: Point,
    pub(crate) layer: Layer,
    pub(crate) box_type: DataType,
}

impl GdsBox {
    /// Creates a new box from two diagonal corner points, layer, and box type.
    /// The corners are normalised so that `bottom_left` holds the minimum
    /// coordinates and `top_right` holds the maximum.
    pub fn new(corner1: Point, corner2: Point, layer: Layer, box_type: DataType) -> Self {
        let (bottom_left, top_right) = Self::normalise_corners(corner1, corner2);
        Self {
            bottom_left,
            top_right,
            layer,
            box_type,
        }
    }

    fn normalise_corners(a: Point, b: Point) -> (Point, Point) {
        crate::geometry::bounding_box(&[a, b])
    }

    /// Returns the bottom-left corner.
    pub const fn bottom_left(&self) -> Point {
        self.bottom_left
    }

    /// Returns the top-right corner.
    pub const fn top_right(&self) -> Point {
        self.top_right
    }

    /// Returns the top-left corner.
    pub fn top_left(&self) -> Point {
        Point::new(self.bottom_left.x(), self.top_right.y())
    }

    /// Returns the bottom-right corner.
    pub fn bottom_right(&self) -> Point {
        Point::new(self.top_right.x(), self.bottom_left.y())
    }

    /// Returns the centre point of the box.
    pub fn center(&self) -> Point {
        (self.bottom_left + self.top_right) * 0.5
    }

    /// Returns the 5 closed points (bottom-left, bottom-right, top-right, top-left, bottom-left)
    /// as expected by the GDS format.
    pub fn points(&self) -> [Point; 5] {
        [
            self.bottom_left,
            self.bottom_right(),
            self.top_right,
            self.top_left(),
            self.bottom_left,
        ]
    }

    /// Returns the layer number.
    pub const fn layer(&self) -> Layer {
        self.layer
    }

    /// Returns the box type.
    pub const fn box_type(&self) -> DataType {
        self.box_type
    }

    /// Remaps the layer and box type using the given mapping.
    /// If the current (layer, `box_type`) pair is found in the mapping, it is replaced.
    pub fn remap_layers(&mut self, mapping: &LayerMapping) {
        if let Some(&(new_layer, new_box_type)) = mapping.get(&(self.layer, self.box_type)) {
            self.layer = new_layer;
            self.box_type = new_box_type;
        }
    }

    /// Converts all points to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            bottom_left: self.bottom_left.to_integer_unit(),
            top_right: self.top_right.to_integer_unit(),
            ..self
        }
    }

    /// Converts all points to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            bottom_left: self.bottom_left.to_float_unit(),
            top_right: self.top_right.to_float_unit(),
            ..self
        }
    }
}

impl std::fmt::Display for GdsBox {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Box from ({}, {}) to ({}, {}) on layer {}, box type {}",
            self.bottom_left.x(),
            self.bottom_left.y(),
            self.top_right.x(),
            self.top_right.y(),
            self.layer(),
            self.box_type()
        )
    }
}

impl Transformable for GdsBox {
    fn transform_impl(self, transformation: &crate::Transformation) -> Self {
        let corners = [
            self.bottom_left.transform(transformation),
            self.top_right.transform(transformation),
            self.top_left().transform(transformation),
            self.bottom_right().transform(transformation),
        ];
        let (new_min, new_max) = crate::geometry::bounding_box(&corners);
        Self {
            bottom_left: new_min,
            top_right: new_max,
            ..self
        }
    }
}

impl Movable for GdsBox {
    fn move_to(self, target: Point) -> Self {
        let delta = target - self.bottom_left;
        self.move_by(delta)
    }
}

impl Dimensions for GdsBox {
    fn bounding_box(&self) -> (Point, Point) {
        (self.bottom_left, self.top_right)
    }
}

impl ToGds for GdsBox {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        validate_layer(self.layer())?;
        validate_data_type(self.box_type())?;

        let mut buffer = Vec::new();

        let box_head = [
            4,
            combine_record_and_data_type(GDSRecord::Box, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer().value(),
            6,
            combine_record_and_data_type(GDSRecord::BoxType, GDSDataType::TwoByteSignedInteger),
            self.box_type().value(),
        ];

        write_u16_array_to_file(&mut buffer, &box_head)?;

        let points = self.points();
        write_points_to_file(&mut buffer, &points, database_units)?;

        write_element_tail_to_file(&mut buffer)?;

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: i32, y: i32) -> Point {
        Point::integer(x, y, 1e-9)
    }

    #[test]
    fn creation_normalises_corners() {
        let gds_box = GdsBox::new(p(10, 10), p(0, 0), Layer::new(1), DataType::new(0));
        assert_eq!(gds_box.bottom_left(), p(0, 0));
        assert_eq!(gds_box.top_right(), p(10, 10));
        assert_eq!(gds_box.layer(), Layer::new(1));
        assert_eq!(gds_box.box_type(), DataType::new(0));
    }

    #[test]
    fn default_is_zero() {
        let gds_box = GdsBox::default();
        assert_eq!(gds_box.bottom_left(), Point::default());
        assert_eq!(gds_box.top_right(), Point::default());
        assert_eq!(gds_box.layer(), Layer::default());
        assert_eq!(gds_box.box_type(), DataType::default());
    }

    #[test]
    fn corner_accessors() {
        let gds_box = GdsBox::new(p(0, 0), p(10, 20), Layer::new(1), DataType::new(0));

        insta::assert_snapshot!(format!("bottom_left: {}", gds_box.bottom_left()), @"bottom_left: Point(0.000000 (1.000e-9), 0.000000 (1.000e-9))");
        insta::assert_snapshot!(format!("bottom_right: {}", gds_box.bottom_right()), @"bottom_right: Point(10.000000 (1.000e-9), 0.000000 (1.000e-9))");
        insta::assert_snapshot!(format!("top_left: {}", gds_box.top_left()), @"top_left: Point(0.000000 (1.000e-9), 20.000000 (1.000e-9))");
        insta::assert_snapshot!(format!("top_right: {}", gds_box.top_right()), @"top_right: Point(10.000000 (1.000e-9), 20.000000 (1.000e-9))");
    }

    #[test]
    fn center() {
        let gds_box = GdsBox::new(p(0, 0), p(10, 20), Layer::new(1), DataType::new(0));
        insta::assert_snapshot!(format!("center: {}", gds_box.center()), @"center: Point(5.000000 (1.000e-9), 10.000000 (1.000e-9))");
    }

    #[test]
    fn display() {
        let gds_box = GdsBox::new(p(0, 0), p(5, 5), Layer::new(2), DataType::new(1));
        insta::assert_snapshot!(gds_box.to_string(), @"Box from (0.000000 (1.000e-9), 0.000000 (1.000e-9)) to (5.000000 (1.000e-9), 5.000000 (1.000e-9)) on layer 2, box type 1");
    }

    #[test]
    fn points_are_closed_rectangle() {
        let gds_box = GdsBox::new(p(0, 0), p(10, 20), Layer::new(1), DataType::new(0));
        let pts = gds_box.points();
        assert_eq!(pts.len(), 5);
        assert_eq!(pts[0], pts[4]);
        assert_eq!(pts[0], p(0, 0));
        assert_eq!(pts[1], p(10, 0));
        assert_eq!(pts[2], p(10, 20));
        assert_eq!(pts[3], p(0, 20));
    }

    #[test]
    fn bounding_box() {
        let gds_box = GdsBox::new(p(0, 0), p(10, 10), Layer::new(1), DataType::new(0));
        let (min, max) = gds_box.bounding_box();
        assert_eq!(min, p(0, 0));
        assert_eq!(max, p(10, 10));
    }

    #[test]
    fn move_to() {
        let gds_box = GdsBox::new(p(0, 0), p(10, 10), Layer::new(1), DataType::new(0));
        let moved = gds_box.move_to(p(5, 5));
        assert_eq!(moved.bottom_left(), p(5, 5));
        assert_eq!(moved.top_right(), p(15, 15));
    }

    #[test]
    fn to_integer_unit() {
        let gds_box = GdsBox::new(
            Point::float(1.5, 2.5, 1e-6),
            Point::float(10.0, 10.0, 1e-6),
            Layer::new(1),
            DataType::new(0),
        );
        let converted = gds_box.to_integer_unit();
        assert_eq!(
            converted.bottom_left(),
            converted.bottom_left().to_integer_unit()
        );
        assert_eq!(
            converted.top_right(),
            converted.top_right().to_integer_unit()
        );
    }

    #[test]
    fn to_float_unit() {
        let gds_box = GdsBox::new(p(0, 0), p(10, 10), Layer::new(1), DataType::new(0));
        let converted = gds_box.to_float_unit();
        assert_eq!(
            converted.bottom_left(),
            converted.bottom_left().to_float_unit()
        );
        assert_eq!(converted.top_right(), converted.top_right().to_float_unit());
    }
}
