use crate::{DataType, Layer, Movable, Point, Transformable};

mod io;
mod path_type;

pub use path_type::PathType;

pub type Width = f64;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Path {
    pub points: Vec<Point>,
    pub layer: Layer,
    pub data_type: DataType,
    pub r#type: Option<PathType>,
    pub width: Option<Width>,
}

impl Path {
    pub fn new(
        points: impl IntoIterator<Item = Point>,
        layer: Layer,
        data_type: DataType,
        path_type: Option<PathType>,
        width: Option<Width>,
    ) -> Self {
        Self {
            points: points.into_iter().collect(),
            layer,
            data_type,
            r#type: path_type,
            width,
        }
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    pub const fn layer(&self) -> Layer {
        self.layer
    }

    pub const fn data_type(&self) -> DataType {
        self.data_type
    }

    pub const fn path_type(&self) -> &Option<PathType> {
        &self.r#type
    }

    pub const fn width(&self) -> Option<Width> {
        self.width
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
    fn transform_impl(&self, transformation: &crate::Transformation) -> Self {
        let points = self
            .points()
            .iter()
            .map(|point| point.transform(transformation));

        Self::new(
            points,
            self.layer(),
            self.data_type(),
            *self.path_type(),
            self.width(),
        )
    }
}

impl Movable for Path {
    fn move_to(&self, target: Point) -> Self {
        let first_point = &self.points()[0];
        let delta = target - *first_point;
        self.move_by(delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(100, 100, 1e-9)];
        let path = Path::new(points.clone(), 1, 2, Some(PathType::Round), Some(10.0));

        assert_eq!(path.points(), &points);
        assert_eq!(path.layer(), 1);
        assert_eq!(path.data_type(), 2);
        assert_eq!(path.path_type(), &Some(PathType::Round));
        assert_eq!(path.width(), Some(10.0));
    }

    #[test]
    fn test_path_default() {
        let path = Path::default();

        assert!(path.points().is_empty());
        assert_eq!(path.layer(), 0);
        assert_eq!(path.data_type(), 0);
        assert_eq!(path.path_type(), &None);
        assert_eq!(path.width(), None);
    }

    #[test]
    fn test_path_display() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(100, 100, 1e-9)];
        let path = Path::new(points, 5, 10, Some(PathType::Square), Some(20.0));

        let display_str = format!("{path}");
        assert!(display_str.contains("Path with 2 points"));
        assert!(display_str.contains("layer 5"));
        assert!(display_str.contains("data type 10"));
        assert!(display_str.contains("Square"));
        assert!(display_str.contains("width 20"));
    }

    #[test]
    fn test_path_clone_and_partial_eq() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(10, 10, 1e-9)];
        let path1 = Path::new(points.clone(), 1, 2, Some(PathType::Round), Some(5.0));
        let path2 = path1.clone();

        assert_eq!(path1, path2);

        let path3 = Path::new(points, 1, 2, Some(PathType::Square), Some(5.0));
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_path_with_different_unit_points() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::float(100.0, 100.0, 1e-6)];
        let path = Path::new(points, 0, 0, None, None);
        assert_eq!(path.points().len(), 2);
    }
}
