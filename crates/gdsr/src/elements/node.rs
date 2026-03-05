use crate::config::gds_file_types::{GDSDataType, GDSRecord, combine_record_and_data_type};
use crate::error::GdsError;
use crate::traits::ToGds;
use crate::utils::io::{
    validate_data_type, validate_layer, write_element_tail_to_file, write_points_to_file,
    write_u16_array_to_file,
};
use crate::{DataType, Dimensions, Layer, Movable, Point, Transformable};

/// Maximum number of points allowed in a GDS II Node element.
pub const MAX_NODE_POINTS: usize = 50;

/// A GDS II Node element used as a point-like marker for design verification (DRC/LVS).
///
/// Nodes contain 1–50 points on a specific layer with a node type qualifier.
#[derive(Clone, Debug, PartialEq, Default)]
#[expect(clippy::struct_field_names)]
pub struct Node {
    pub(crate) points: Vec<Point>,
    pub(crate) layer: Layer,
    pub(crate) node_type: DataType,
}

impl Node {
    /// Creates a new node from points, layer, and node type.
    /// Points beyond 50 are silently truncated to comply with the GDS spec.
    pub fn new(points: impl Into<Vec<Point>>, layer: Layer, node_type: DataType) -> Self {
        let mut points = points.into();
        points.truncate(MAX_NODE_POINTS);
        Self {
            points,
            layer,
            node_type,
        }
    }

    /// Returns the points in this node.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Returns the layer number.
    pub const fn layer(&self) -> Layer {
        self.layer
    }

    /// Returns the node type.
    pub const fn node_type(&self) -> DataType {
        self.node_type
    }

    /// Converts all points to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            points: self
                .points
                .into_iter()
                .map(|p| p.to_integer_unit())
                .collect(),
            ..self
        }
    }

    /// Converts all points to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            points: self.points.into_iter().map(|p| p.to_float_unit()).collect(),
            ..self
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node with {} point(s) on layer {}, node type {}",
            self.points.len(),
            self.layer(),
            self.node_type()
        )
    }
}

impl Transformable for Node {
    fn transform_impl(self, transformation: &crate::Transformation) -> Self {
        Self {
            points: self
                .points
                .into_iter()
                .map(|p| p.transform(transformation))
                .collect(),
            ..self
        }
    }
}

impl Movable for Node {
    fn move_to(self, target: Point) -> Self {
        if let Some(&first) = self.points.first() {
            let delta = target - first;
            self.move_by(delta)
        } else {
            self
        }
    }
}

impl Dimensions for Node {
    fn bounding_box(&self) -> (Point, Point) {
        crate::geometry::bounding_box(&self.points)
    }
}

impl ToGds for Node {
    fn to_gds_impl(&self, database_units: f64) -> Result<Vec<u8>, GdsError> {
        validate_layer(self.layer())?;
        validate_data_type(self.node_type())?;

        let mut buffer = Vec::new();

        let node_head = [
            4,
            combine_record_and_data_type(GDSRecord::Node, GDSDataType::NoData),
            6,
            combine_record_and_data_type(GDSRecord::Layer, GDSDataType::TwoByteSignedInteger),
            self.layer().value(),
            6,
            combine_record_and_data_type(GDSRecord::NodeType, GDSDataType::TwoByteSignedInteger),
            self.node_type().value(),
        ];

        write_u16_array_to_file(&mut buffer, &node_head)?;
        write_points_to_file(&mut buffer, &self.points, database_units)?;
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
    fn creation() {
        let node = Node::new(vec![p(1, 2), p(3, 4)], Layer::new(5), DataType::new(7));
        assert_eq!(node.points().len(), 2);
        assert_eq!(node.layer(), Layer::new(5));
        assert_eq!(node.node_type(), DataType::new(7));
    }

    #[test]
    fn truncates_beyond_max_points() {
        let points: Vec<Point> = (0..60).map(|i| p(i, i)).collect();
        let node = Node::new(points, Layer::new(0), DataType::new(0));
        assert_eq!(node.points().len(), MAX_NODE_POINTS);
    }

    #[test]
    fn default_is_empty() {
        let node = Node::default();
        assert!(node.points().is_empty());
        assert_eq!(node.layer(), Layer::default());
        assert_eq!(node.node_type(), DataType::default());
    }

    #[test]
    fn display() {
        let node = Node::new(vec![p(0, 0), p(5, 5)], Layer::new(2), DataType::new(1));
        insta::assert_snapshot!(node.to_string(), @"Node with 2 point(s) on layer 2, node type 1");
    }

    #[test]
    fn bounding_box() {
        let node = Node::new(
            vec![p(0, 0), p(10, 5), p(3, 20)],
            Layer::new(1),
            DataType::new(0),
        );
        let (min, max) = node.bounding_box();
        assert_eq!(min, p(0, 0));
        assert_eq!(max, p(10, 20));
    }

    #[test]
    fn move_to() {
        let node = Node::new(vec![p(0, 0), p(10, 10)], Layer::new(1), DataType::new(0));
        let moved = node.move_to(p(5, 5));
        assert_eq!(moved.points()[0], p(5, 5));
        assert_eq!(moved.points()[1], p(15, 15));
    }

    #[test]
    fn move_to_empty_is_noop() {
        let node = Node::new(Vec::<Point>::new(), Layer::new(1), DataType::new(0));
        let moved = node.clone().move_to(p(5, 5));
        assert_eq!(moved, node);
    }

    #[test]
    fn to_integer_unit() {
        let node = Node::new(
            vec![Point::float(1.5, 2.5, 1e-6), Point::float(10.0, 10.0, 1e-6)],
            Layer::new(1),
            DataType::new(0),
        );
        let converted = node.to_integer_unit();
        for point in converted.points() {
            assert_eq!(*point, point.to_integer_unit());
        }
    }

    #[test]
    fn to_float_unit() {
        let node = Node::new(vec![p(0, 0), p(10, 10)], Layer::new(1), DataType::new(0));
        let converted = node.to_float_unit();
        for point in converted.points() {
            assert_eq!(*point, point.to_float_unit());
        }
    }
}
