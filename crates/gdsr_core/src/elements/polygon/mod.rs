use crate::{
    CoordNum, DataType, DatabaseIntegerUnit, Layer, Point,
    traits::{Dimensions, Movable, Transformable},
    transformation::Transformation,
    utils::geometry::{area, bounding_box, is_point_inside, is_point_on_edge, perimeter},
};

mod io;
mod utils;

#[derive(Clone, Debug, PartialEq)]
pub struct Polygon<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub(crate) points: Vec<Point<DatabaseUnitT>>,
    pub(crate) layer: Layer,
    pub(crate) data_type: DataType,
}

impl<DatabaseUnitT: CoordNum> Default for Polygon<DatabaseUnitT> {
    fn default() -> Self {
        Self {
            points: Vec::default(),
            layer: Default::default(),
            data_type: Default::default(),
        }
    }
}

impl<DatabaseUnitT: CoordNum> Polygon<DatabaseUnitT> {
    #[must_use]
    pub fn new(
        points: impl IntoIterator<Item = impl Into<Point<DatabaseUnitT>>>,
        layer: Layer,
        data_type: DataType,
    ) -> Self {
        Self {
            points: utils::get_correct_polygon_points_format(points),
            layer,
            data_type,
        }
    }

    #[must_use]
    pub fn points(&self) -> &[Point<DatabaseUnitT>] {
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
    pub fn area(&self) -> DatabaseUnitT {
        area(&self.points)
    }

    #[must_use]
    pub fn perimeter(&self) -> DatabaseUnitT {
        perimeter(&self.points)
    }

    pub fn is_point_inside(&self, point: &Point<DatabaseUnitT>) -> bool {
        is_point_inside(point, &self.points)
    }

    pub fn is_point_on_edge(&self, point: &Point<DatabaseUnitT>) -> bool {
        is_point_on_edge(point, &self.points)
    }
}

impl<DatabaseUnitT: CoordNum> std::fmt::Display for Polygon<DatabaseUnitT> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

impl<DatabaseUnitT: CoordNum> Transformable for Polygon<DatabaseUnitT> {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.points = new_self
            .points
            .iter()
            .map(|point| transformation.apply_to_point(point))
            .collect();
        new_self
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Polygon<DatabaseUnitT> {
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self {
        let first_point = &self.points()[0];
        let delta = Point::new(
            DatabaseIntegerUnit::from_float(target.x().to_float() - first_point.x().to_float()),
            DatabaseIntegerUnit::from_float(target.y().to_float() - first_point.y().to_float()),
        );
        self.move_by(delta)
    }
}

impl<DatabaseUnitT: CoordNum> Dimensions<DatabaseUnitT> for Polygon<DatabaseUnitT> {
    fn bounding_box(&self) -> (Point<DatabaseUnitT>, Point<DatabaseUnitT>) {
        bounding_box(self.points())
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    use crate::{
        DatabaseIntegerUnit,
        transformation::{Reflection, Rotation, Scale, Translation},
    };

    fn create_square() -> Polygon<DatabaseIntegerUnit> {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(10), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(10), DatabaseIntegerUnit::from(10)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(10)),
        ];
        Polygon::new(points, 1, 0)
    }

    fn create_triangle() -> Polygon<DatabaseIntegerUnit> {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(10)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
        ];
        Polygon::new(points, 2, 1)
    }

    #[test]
    fn test_default() {
        let polygon: Polygon<DatabaseIntegerUnit> = Polygon::default();
        assert_eq!(polygon.points().len(), 0);
        assert_eq!(polygon.layer(), 0);
        assert_eq!(polygon.data_type(), 0);
    }

    #[test]
    fn test_new() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(5)),
        ];
        let polygon = Polygon::new(points.clone(), 3, 2);

        assert_eq!(polygon.layer(), 3);
        assert_eq!(polygon.data_type(), 2);
        assert_eq!(polygon.points().len(), 3);
        assert_eq!(polygon.points()[0], points[0]);
        assert_eq!(polygon.points()[1], points[1]);
        assert_eq!(polygon.points()[2], points[0]); // Closed
    }

    #[test]
    fn test_getters() {
        let polygon = create_square();
        assert_eq!(polygon.layer(), 1);
        assert_eq!(polygon.data_type(), 0);
        assert_eq!(polygon.points().len(), 5); // 4 + 1 to close
    }

    #[test]
    fn test_area_square() {
        let polygon = create_square();
        let expected_area = DatabaseIntegerUnit::from(100); // 10 * 10
        assert_eq!(polygon.area(), expected_area);
    }

    #[test]
    fn test_area_triangle() {
        let polygon = create_triangle();
        let expected_area = DatabaseIntegerUnit::from(25); // 0.5 * base * height = 0.5 * 10 * 5
        assert_eq!(polygon.area(), expected_area);
    }

    #[test]
    fn test_perimeter_square() {
        let polygon = create_square();
        let expected_perimeter = DatabaseIntegerUnit::from(40); // 4 * 10
        assert_eq!(polygon.perimeter(), expected_perimeter);
    }

    #[test]
    fn test_is_point_inside() {
        let polygon = create_square();

        let inside_point = Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(5));
        assert!(polygon.is_point_inside(&inside_point));

        let outside_point = Point::new(DatabaseIntegerUnit::from(15), DatabaseIntegerUnit::from(5));
        assert!(!polygon.is_point_inside(&outside_point));

        let vertex_point = Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0));
        assert!(polygon.is_point_inside(&vertex_point) || polygon.is_point_on_edge(&vertex_point));
    }

    #[test]
    fn test_is_point_on_edge() {
        let polygon = create_square();

        let edge_point = Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(0));
        assert!(polygon.is_point_on_edge(&edge_point));

        let not_on_edge = Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(5));
        assert!(!polygon.is_point_on_edge(&not_on_edge));

        let vertex_point = Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0));
        assert!(polygon.is_point_on_edge(&vertex_point));
    }

    #[test]
    fn test_display() {
        let polygon = create_square();
        let display_str = format!("{polygon}");
        assert!(display_str.contains("Polygon with 5 point(s)"));
        assert!(display_str.contains("starting at (0, 0)"));
        assert!(display_str.contains("layer 1"));
        assert!(display_str.contains("data type 0"));
    }

    #[test]
    fn test_clone() {
        let polygon = create_square();
        let cloned = polygon.clone();
        assert_eq!(polygon.points(), cloned.points());
        assert_eq!(polygon.layer(), cloned.layer());
        assert_eq!(polygon.data_type(), cloned.data_type());
    }

    #[test]
    fn test_partial_eq() {
        let polygon1 = create_square();
        let polygon2 = create_square();
        let polygon3 = create_triangle();

        assert_eq!(polygon1, polygon2);
        assert_ne!(polygon1, polygon3);
    }

    #[test]
    fn test_transformable_translation() {
        let polygon = create_square();
        let translation = Translation::new(Point::new(
            DatabaseIntegerUnit::from(5),
            DatabaseIntegerUnit::from(3),
        ));

        let transformed = polygon.transform(translation);

        assert_eq!(transformed.points()[0].x(), DatabaseIntegerUnit::from(5));
        assert_eq!(transformed.points()[0].y(), DatabaseIntegerUnit::from(3));
    }

    #[test]
    fn test_transformable_rotation() {
        let polygon = create_square();
        let rotation = Rotation::new(PI / 2.0, Point::new(0, 0));

        let transformed = polygon.transform(rotation);

        assert_eq!(transformed.points().len(), polygon.points().len());
        assert_eq!(transformed.area(), polygon.area());
    }

    #[test]
    fn test_transformable_scale() {
        let polygon = create_square();
        let scale = Scale::new(
            2.0,
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
        );

        let transformed = polygon.transform(scale);

        assert_eq!(
            transformed.area(),
            polygon.area() * DatabaseIntegerUnit::from(4)
        );
    }

    #[test]
    fn test_transformable_reflection() {
        let polygon = create_square();
        let reflection = Reflection::new_horizontal();

        let transformed = polygon.transform(reflection);

        assert_eq!(transformed.area(), polygon.area());
        assert_eq!(transformed.points().len(), polygon.points().len());
    }

    #[test]
    fn test_movable_move_to() {
        let polygon = create_square();
        let target = Point::new(
            DatabaseIntegerUnit::from(100),
            DatabaseIntegerUnit::from(200),
        );

        let moved = polygon.move_to(target);

        assert_eq!(moved.points()[0].x(), DatabaseIntegerUnit::from(100));
        assert_eq!(moved.points()[0].y(), DatabaseIntegerUnit::from(200));

        assert_eq!(moved.area(), polygon.area());
    }

    #[test]
    fn test_dimensions_bounding_box() {
        let polygon = create_square();
        let (min_point, max_point) = polygon.bounding_box();

        assert_eq!(min_point.x(), DatabaseIntegerUnit::from(0));
        assert_eq!(min_point.y(), DatabaseIntegerUnit::from(0));
        assert_eq!(max_point.x(), DatabaseIntegerUnit::from(10));
        assert_eq!(max_point.y(), DatabaseIntegerUnit::from(10));
    }

    #[test]
    fn test_empty_polygon() {
        let empty_points: Vec<Point<DatabaseIntegerUnit>> = vec![];
        let polygon = Polygon::new(empty_points, 0, 0);

        assert_eq!(polygon.points().len(), 0);
        assert_eq!(polygon.area(), DatabaseIntegerUnit::from(0));
    }

    #[test]
    fn test_single_point_polygon() {
        let single_point = vec![Point::new(
            DatabaseIntegerUnit::from(5),
            DatabaseIntegerUnit::from(5),
        )];
        let polygon = Polygon::new(single_point, 0, 0);

        assert_eq!(polygon.points().len(), 1);
        assert_eq!(polygon.area(), DatabaseIntegerUnit::from(0));
    }

    #[test]
    fn test_already_closed_polygon() {
        let points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(5), DatabaseIntegerUnit::from(5)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
        ];
        let polygon = Polygon::new(points.clone(), 0, 0);

        assert_eq!(polygon.points().len(), points.len());
    }

    #[test]
    fn test_transformation_rotation_and_scale_area_and_points() {
        let polygon = create_square();
        let original_area = polygon.area();

        let rotation = Rotation::new(PI / 2.0, Point::new(0, 0));
        let scale = Scale::new(2.0, Point::new(0, 0));

        let mut transformation = Transformation::from(rotation);
        let transformation = transformation.with_scale(Some(scale));
        let transformed = polygon.transform(transformation);

        let expected_area = original_area * DatabaseIntegerUnit::from(4);
        assert_eq!(transformed.area(), expected_area);

        let expected_points = vec![
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(20)),
            Point::new(
                DatabaseIntegerUnit::from(-20),
                DatabaseIntegerUnit::from(20),
            ),
            Point::new(DatabaseIntegerUnit::from(-20), DatabaseIntegerUnit::from(0)),
            Point::new(DatabaseIntegerUnit::from(0), DatabaseIntegerUnit::from(0)),
        ];

        assert_eq!(transformed.points(), expected_points);
    }
}
