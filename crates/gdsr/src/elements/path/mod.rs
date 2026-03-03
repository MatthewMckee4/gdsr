use crate::{DataType, Layer, Movable, Point, Transformable, Unit};

mod io;
mod path_type;

pub use path_type::PathType;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(100, 100, 1e-9)];
        let path = Path::new(
            points.clone(),
            1,
            2,
            Some(PathType::Round),
            Some(Unit::default_integer(10)),
        );

        assert_eq!(path.points(), &points);
        assert_eq!(path.layer(), 1);
        assert_eq!(path.data_type(), 2);
        assert_eq!(path.path_type(), &Some(PathType::Round));
        assert_eq!(path.width(), Some(Unit::default_integer(10)));
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
        let path = Path::new(
            points,
            5,
            10,
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
            1,
            2,
            Some(PathType::Round),
            Some(Unit::default_integer(5)),
        );
        let path2 = path1.clone();

        assert_eq!(path1, path2);

        let path3 = Path::new(
            points,
            1,
            2,
            Some(PathType::Square),
            Some(Unit::default_integer(5)),
        );
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_path_with_different_unit_points() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::float(100.0, 100.0, 1e-6)];
        let path = Path::new(points, 0, 0, None, None);
        assert_eq!(path.points().len(), 2);
    }
}
