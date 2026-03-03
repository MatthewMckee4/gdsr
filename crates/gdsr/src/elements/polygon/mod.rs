use crate::{DataType, Dimensions, Layer, Movable, Point, Transformable, Unit};

mod io;
mod utils;

#[cfg(test)]
pub(crate) use utils::close_points;
pub use utils::get_correct_polygon_points_format;

/// A closed polygon defined by a sequence of points on a specific layer.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Polygon {
    pub(crate) points: Vec<Point>,
    pub(crate) layer: Layer,
    pub(crate) data_type: DataType,
}

impl Polygon {
    /// Creates a new polygon from the given points, layer, and data type.
    /// The polygon is automatically closed if needed.
    pub fn new(points: impl IntoIterator<Item = Point>, layer: Layer, data_type: DataType) -> Self {
        Self {
            points: utils::get_correct_polygon_points_format(points),
            layer,
            data_type,
        }
    }

    /// Returns the polygon's points (including the closing point).
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

    /// Computes the area of the polygon using the shoelace formula.
    pub fn area(&self) -> Unit {
        crate::utils::geometry::area(&self.points)
    }

    /// Computes the perimeter of the polygon.
    pub fn perimeter(&self) -> Unit {
        crate::utils::geometry::perimeter(&self.points)
    }

    /// Check if a point is inside the polygon
    pub fn is_point_inside(&self, point: &Point) -> bool {
        crate::utils::geometry::is_point_inside(point, &self.points)
    }

    /// Converts all points to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            points: self.points.iter().map(Point::to_integer_unit).collect(),
            ..self
        }
    }

    /// Converts all points to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            points: self.points.iter().map(Point::to_float_unit).collect(),
            ..self
        }
    }

    /// Check if a point lies on the edge of the polygon
    pub fn is_point_on_edge(&self, point: &Point) -> bool {
        crate::utils::geometry::is_point_on_edge(point, &self.points)
    }
}

impl std::fmt::Display for Polygon {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.points().is_empty() {
            write!(
                f,
                "Polygon with 0 points on layer {:?}, data type {:?}",
                self.layer(),
                self.data_type()
            )
        } else {
            write!(
                f,
                "Polygon with {} point(s), starting at ({}, {}) on layer {:?}, data type {:?}",
                self.points().len(),
                self.points()[0].x(),
                self.points()[0].y(),
                self.layer(),
                self.data_type()
            )
        }
    }
}

impl Transformable for Polygon {
    fn transform_impl(mut self, transformation: &crate::Transformation) -> Self {
        self.points = self
            .points()
            .iter()
            .map(|point| point.transform(transformation))
            .collect();

        self
    }
}

impl Movable for Polygon {
    fn move_to(self, target: Point) -> Self {
        let Some(first_point) = self.points().first() else {
            return self;
        };
        let delta = target - *first_point;
        self.move_by(delta)
    }
}

impl Dimensions for Polygon {
    fn bounding_box(&self) -> (Point, Point) {
        crate::utils::geometry::bounding_box(&self.points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon_creation() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);

        assert_eq!(polygon.layer(), 1);
        assert_eq!(polygon.data_type(), 0);
        // Should be closed automatically
        assert_eq!(polygon.points().len(), 4);
    }

    #[test]
    fn test_polygon_default() {
        let polygon = Polygon::default();
        assert_eq!(polygon.points().len(), 0);
        assert_eq!(polygon.layer(), 0);
        assert_eq!(polygon.data_type(), 0);
    }

    #[test]
    fn test_polygon_display() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(5, 0, 1e-9),
            Point::integer(5, 5, 1e-9),
        ];
        let polygon = Polygon::new(points, 2, 1);

        insta::assert_snapshot!(polygon.to_string(), @"Polygon with 4 point(s), starting at (0 (1.000e-9), 0 (1.000e-9)) on layer 2, data type 1");
    }

    #[test]
    fn test_polygon_display_empty() {
        let polygon = Polygon::new(vec![], 1, 0);
        insta::assert_snapshot!(polygon.to_string());
    }

    #[test]
    fn test_polygon_area() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
            Point::integer(0, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        assert_eq!(polygon.area(), Unit::float(100.0, 1e-9));

        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(5, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        assert_eq!(polygon.area(), Unit::float(50.0, 1e-9));

        let polygon = Polygon::new(vec![], 1, 0);
        assert_eq!(polygon.area(), Unit::float(0.0, 1e-9));
    }

    #[test]
    fn test_polygon_perimeter() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
            Point::integer(0, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        assert_eq!(polygon.perimeter(), Unit::float(40.0, 1e-9));

        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(3, 0, 1e-9),
            Point::integer(0, 4, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        assert_eq!(polygon.perimeter(), Unit::float(12.0, 1e-9));

        let polygon = Polygon::new(vec![], 1, 0);
        assert_eq!(polygon.perimeter(), Unit::float(0.0, 1e-9));
    }

    #[test]
    fn test_is_point_inside() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
            Point::integer(0, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);

        assert!(polygon.is_point_inside(&Point::integer(5, 5, 1e-9)));
        assert!(!polygon.is_point_inside(&Point::integer(15, 15, 1e-9)));
        assert!(!polygon.is_point_inside(&Point::integer(-5, 5, 1e-9)));
    }

    #[test]
    fn test_is_point_on_edge() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
            Point::integer(0, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);

        assert!(polygon.is_point_on_edge(&Point::integer(5, 0, 1e-9)));
        assert!(polygon.is_point_on_edge(&Point::integer(10, 5, 1e-9)));
        assert!(polygon.is_point_on_edge(&Point::integer(0, 0, 1e-9)));
        assert!(!polygon.is_point_on_edge(&Point::integer(5, 5, 1e-9)));
        assert!(!polygon.is_point_on_edge(&Point::integer(15, 15, 1e-9)));
        assert!(!polygon.is_point_on_edge(&Point::integer(-5, 5, 1e-9)));
    }

    #[test]
    fn test_polygon_bounding_box() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
            Point::integer(0, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        let (min, max) = polygon.bounding_box();
        assert_eq!(min, Point::integer(0, 0, 1e-9));
        assert_eq!(max, Point::integer(10, 10, 1e-9));

        let points = vec![
            Point::integer(-5, -3, 1e-9),
            Point::integer(7, 2, 1e-9),
            Point::integer(3, 8, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        let (min, max) = polygon.bounding_box();
        assert_eq!(min, Point::integer(-5, -3, 1e-9));
        assert_eq!(max, Point::integer(7, 8, 1e-9));
    }

    #[test]
    fn test_polygon_move_to() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
            Point::integer(0, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        let target = Point::integer(5, 5, 1e-9);
        let moved = polygon.move_to(target);

        assert_eq!(moved.points()[0], Point::integer(5, 5, 1e-9));
        assert_eq!(moved.points()[1], Point::integer(15, 5, 1e-9));
        assert_eq!(moved.points()[2], Point::integer(15, 15, 1e-9));
        assert_eq!(moved.points()[3], Point::integer(5, 15, 1e-9));
        assert_eq!(moved.points()[4], Point::integer(5, 5, 1e-9));
    }

    #[test]
    fn test_polygon_to_integer_unit() {
        let points = vec![
            Point::float(1.5, 2.5, 1e-6),
            Point::float(10.0, 0.0, 1e-6),
            Point::float(10.0, 10.0, 1e-6),
        ];
        let polygon = Polygon::new(points, 1, 0);
        let converted = polygon.to_integer_unit();

        for point in converted.points() {
            assert_eq!(*point, point.to_integer_unit());
        }
    }

    #[test]
    fn test_polygon_to_float_unit() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
        ];
        let polygon = Polygon::new(points, 1, 0);
        let converted = polygon.to_float_unit();

        for point in converted.points() {
            assert_eq!(*point, point.to_float_unit());
        }
    }

    #[test]
    fn test_polygon_move_to_empty() {
        let polygon = Polygon::new(vec![], 1, 0);
        let moved = polygon.move_to(Point::integer(5, 5, 1e-9));
        assert_eq!(moved.points().len(), 0);
    }
}
