use crate::{
    CoordNum, DataType, DatabaseFloatUnit, DatabaseIntegerUnit, Layer, Point,
    traits::{Dimensions, Movable, Transformable},
    transformation::Transformation,
    utils::geometry::bounding_box,
};

mod io;
mod path_type;

pub use path_type::PathType;

pub type Width = f64;

#[derive(Clone, Debug, PartialEq)]
pub struct Path<DatabaseUnitT: CoordNum = DatabaseIntegerUnit> {
    pub(crate) points: Vec<Point<DatabaseUnitT>>,
    pub(crate) layer: Layer,
    pub(crate) data_type: DataType,
    pub(crate) r#type: Option<PathType>,
    pub(crate) width: Option<Width>,
}

impl<DatabaseUnitT: CoordNum> Default for Path<DatabaseUnitT> {
    fn default() -> Self {
        Self {
            points: Vec::default(),
            layer: Default::default(),
            data_type: Default::default(),
            r#type: None,
            width: None,
        }
    }
}

impl<DatabaseUnitT: CoordNum> Path<DatabaseUnitT> {
    #[must_use]
    pub const fn new(
        points: Vec<Point<DatabaseUnitT>>,
        layer: Layer,
        data_type: DataType,
        path_type: Option<PathType>,
        width: Option<Width>,
    ) -> Self {
        Self {
            points,
            layer,
            data_type,
            r#type: path_type,
            width,
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
    pub const fn path_type(&self) -> &Option<PathType> {
        &self.r#type
    }

    #[must_use]
    pub const fn width(&self) -> Option<Width> {
        self.width
    }
}

impl<DatabaseUnitT: CoordNum> std::fmt::Display for Path<DatabaseUnitT> {
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

impl<DatabaseUnitT: CoordNum> Transformable for Path<DatabaseUnitT> {
    fn transform_impl(&self, transformation: &Transformation) -> Self {
        let mut new_self = self.clone();
        new_self.points = new_self
            .points()
            .iter()
            .map(|point| transformation.apply_to_point(point))
            .collect();
        new_self
    }
}

impl<DatabaseUnitT: CoordNum> Movable for Path<DatabaseUnitT> {
    fn move_to(&self, target: Point<DatabaseIntegerUnit>) -> Self {
        let first_point = &self.points()[0];
        let delta = Point::new(
            DatabaseIntegerUnit::from_float(target.x().to_float() - first_point.x().to_float()),
            DatabaseIntegerUnit::from_float(target.y().to_float() - first_point.y().to_float()),
        );
        self.move_by(delta)
    }
}

impl Dimensions<DatabaseFloatUnit> for Path<DatabaseIntegerUnit> {
    fn bounding_box(&self) -> (Point<DatabaseFloatUnit>, Point<DatabaseFloatUnit>) {
        let to_database_float = |point: Point<DatabaseIntegerUnit>| {
            Point::new(
                point.x() as DatabaseFloatUnit,
                point.y() as DatabaseFloatUnit,
            )
        };

        if let Some(width) = self.width()
            && width > 0.0
        {
            // For paths with width, we need to consider the extended points
            let half_width = width / 2.0;

            // Create extended points considering the width
            let mut extended_points = Vec::new();

            for i in 0..self.points().len() {
                let point = to_database_float(self.points()[i]);

                // Add points offset by half-width in perpendicular directions
                if i > 0 {
                    let prev = to_database_float(self.points()[i - 1]);

                    let dx = point.x() - prev.x();
                    let dy = point.y() - prev.y();
                    let len = dx.mul_add(dx, dy * dy).sqrt();
                    if len > 0.0 {
                        let nx = -dy / len * half_width;
                        let ny = dx / len * half_width;
                        extended_points.push(Point::new(point.x() + nx, point.y() + ny));
                        extended_points.push(Point::new(point.x() - nx, point.y() - ny));
                    }
                }

                if i < self.points().len() - 1 {
                    let next = to_database_float(self.points()[i + 1]);

                    let dx = next.x() - point.x();
                    let dy = next.y() - point.y();
                    let len = dx.hypot(dy);
                    if len > 0.0 {
                        let nx = -dy / len * half_width;
                        let ny = dx / len * half_width;
                        extended_points.push(Point::new(point.x() + nx, point.y() + ny));
                        extended_points.push(Point::new(point.x() - nx, point.y() - ny));
                    }
                }
            }

            bounding_box(&extended_points)
        } else {
            // For paths without width, use the standard bounding box
            let (bottom_left, bottom_right) = bounding_box(self.points());

            (
                to_database_float(bottom_left),
                to_database_float(bottom_right),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let points = vec![Point::new(0, 0), Point::new(100, 100)];
        let path = Path::new(points.clone(), 1, 2, Some(PathType::Round), Some(10.0));

        assert_eq!(path.points(), &points);
        assert_eq!(path.layer(), 1);
        assert_eq!(path.data_type(), 2);
        assert_eq!(path.path_type(), &Some(PathType::Round));
        assert_eq!(path.width(), Some(10.0));
    }

    #[test]
    fn test_path_default() {
        let path = Path::<DatabaseIntegerUnit>::default();

        assert!(path.points().is_empty());
        assert_eq!(path.layer(), 0);
        assert_eq!(path.data_type(), 0);
        assert_eq!(path.path_type(), &None);
        assert_eq!(path.width(), None);
    }

    #[test]
    fn test_path_display() {
        let points = vec![Point::new(0, 0), Point::new(100, 100)];
        let path = Path::new(points, 5, 10, Some(PathType::Square), Some(20.0));

        let display_str = format!("{path}");
        assert!(display_str.contains("Path with 2 points"));
        assert!(display_str.contains("layer 5"));
        assert!(display_str.contains("data type 10"));
        assert!(display_str.contains("Square"));
        assert!(display_str.contains("width 20"));
    }

    #[test]
    fn test_path_bounding_box_without_width() {
        let points = vec![Point::new(10, 20), Point::new(30, 40), Point::new(50, 10)];
        let path = Path::new(points, 0, 0, None, None);

        let (bottom_left, top_right) = path.bounding_box();
        assert_eq!(bottom_left.x(), 10.0);
        assert_eq!(bottom_left.y(), 10.0);
        assert_eq!(top_right.x(), 50.0);
        assert_eq!(top_right.y(), 40.0);
    }

    #[test]
    fn test_path_bounding_box_with_width() {
        let points = vec![Point::new(0, 0), Point::new(100, 0)];
        let path = Path::new(points, 0, 0, None, Some(20.0));

        let (bottom_left, top_right) = path.bounding_box();
        // With width 20, the path should extend 10 units in each perpendicular direction
        assert!(bottom_left.y() <= -10.0);
        assert!(top_right.y() >= 10.0);
        assert_eq!(bottom_left.x(), 0.0);
        assert_eq!(top_right.x(), 100.0);
    }

    #[test]
    fn test_path_movable() {
        let points = vec![Point::new(10, 20), Point::new(30, 40)];
        let path = Path::new(points, 0, 0, None, None);

        let moved_path = path.move_to(Point::new(50, 60));
        let expected_delta_x = 50 - 10;
        let expected_delta_y = 60 - 20;

        assert_eq!(moved_path.points()[0], Point::new(50, 60));
        assert_eq!(
            moved_path.points()[1],
            Point::new(30 + expected_delta_x, 40 + expected_delta_y)
        );
    }

    #[test]
    fn test_path_transformable() {
        use crate::transformation::{Transformation, Translation};

        let points = vec![Point::new(0, 0), Point::new(10, 10)];
        let path = Path::new(points, 0, 0, None, None);

        let translation = Translation::new(Point::new(5, 5));
        let transformation = Transformation::from(translation);

        let transformed_path = path.transform_impl(&transformation);
        assert_eq!(transformed_path.points()[0], Point::new(5, 5));
        assert_eq!(transformed_path.points()[1], Point::new(15, 15));
    }

    #[test]
    fn test_path_clone_and_partial_eq() {
        let points = vec![Point::new(0, 0), Point::new(10, 10)];
        let path1 = Path::new(points.clone(), 1, 2, Some(PathType::Round), Some(5.0));
        let path2 = path1.clone();

        assert_eq!(path1, path2);

        let path3 = Path::new(points, 1, 2, Some(PathType::Square), Some(5.0));
        assert_ne!(path1, path3);
    }
}
