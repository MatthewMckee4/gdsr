use crate::{DataType, Dimensions, Layer, Movable, Point, Transformable};

mod io;
mod utils;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Polygon {
    pub(crate) points: Vec<Point>,
    pub(crate) layer: Layer,
    pub(crate) data_type: DataType,
}

impl Polygon {
    #[must_use]
    pub fn new(points: impl IntoIterator<Item = Point>, layer: Layer, data_type: DataType) -> Self {
        Self {
            points: utils::get_correct_polygon_points_format(points),
            layer,
            data_type,
        }
    }

    #[must_use]
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    #[must_use]
    pub const fn layer(&self) -> Layer {
        self.layer
    }

    #[must_use]
    pub const fn data_type(&self) -> DataType {
        self.data_type
    }

    #[must_use]
    pub fn area(&self) -> f64 {
        crate::utils::geometry::area(&self.points)
    }

    #[must_use]
    pub fn perimeter(&self) -> f64 {
        crate::utils::geometry::perimeter(&self.points)
    }

    /// Check if a point is inside the polygon
    #[must_use]
    pub fn is_point_inside(&self, point: &Point) -> bool {
        crate::utils::geometry::is_point_inside(point, &self.points)
    }

    /// Check if a point lies on the edge of the polygon
    #[must_use]
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
                "Polygon with {} point(s), starting at ({:?}, {:?}) on layer {:?}, data type {:?}",
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
    fn transform_impl(&self, transformation: &crate::Transformation) -> Self {
        let points: Vec<Point> = self
            .points()
            .iter()
            .map(|point| point.transform(transformation))
            .collect();

        Self::new(points, self.layer(), self.data_type())
    }
}

impl Movable for Polygon {
    fn move_to(&self, target: Point) -> Self {
        let points: Vec<Point> = self
            .points()
            .iter()
            .map(|point| point.move_to(target))
            .collect();

        Self::new(points, self.layer(), self.data_type())
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

        let display_str = format!("{polygon}");
        assert!(display_str.contains("Polygon with"));
        assert!(display_str.contains("layer 2"));
        assert!(display_str.contains("data type 1"));
    }

    #[test]
    fn test_polygon_clone_and_eq() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 0, 1e-9),
            Point::integer(10, 10, 1e-9),
        ];
        let polygon1 = Polygon::new(points, 1, 0);
        let polygon2 = polygon1.clone();

        assert_eq!(polygon1, polygon2);
    }
}
