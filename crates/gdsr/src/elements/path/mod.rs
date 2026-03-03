use crate::{DataType, Dimensions, Layer, Movable, Point, Transformable, Unit};

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
        crate::utils::geometry::bounding_box(&self.points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck::{Arbitrary, Gen};

    const MAX_VALUE: i32 = 10_000;
    const MIN_PATH_POINTS: usize = 2;
    const MAX_EXTRA_POINTS: usize = 18;

    impl Arbitrary for PathType {
        fn arbitrary(g: &mut Gen) -> Self {
            let types = [Self::Square, Self::Round, Self::Overlap];
            types[usize::arbitrary(g) % types.len()]
        }
    }

    impl Arbitrary for Path {
        fn arbitrary(g: &mut Gen) -> Self {
            let units_options = [1e-9, 1e-8, 1e-7, 1e-6];
            let units = units_options[usize::arbitrary(g) % units_options.len()];
            let num_points = MIN_PATH_POINTS + (usize::arbitrary(g) % (MAX_EXTRA_POINTS + 1));
            let points: Vec<Point> = (0..num_points)
                .map(|_| {
                    let x = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                    let y = (i32::arbitrary(g) % MAX_VALUE).clamp(-MAX_VALUE, MAX_VALUE);
                    Point::integer(x, y, units)
                })
                .collect();
            let layer = u16::arbitrary(g);
            let data_type = u16::arbitrary(g);
            let path_type = if bool::arbitrary(g) {
                Some(PathType::arbitrary(g))
            } else {
                None
            };
            let width = if bool::arbitrary(g) {
                let w = (i32::arbitrary(g) % MAX_VALUE).clamp(0, MAX_VALUE);
                Some(Unit::integer(w, units))
            } else {
                None
            };
            Self::new(points, layer, data_type, path_type, width)
        }
    }

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
    fn test_path_to_integer_unit() {
        let points = vec![Point::float(1.5, 2.5, 1e-6), Point::float(10.0, 10.0, 1e-6)];
        let path = Path::new(
            points,
            1,
            0,
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
        let path = Path::new(points, 1, 0, None, Some(Unit::default_integer(10)));
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
        let path = Path::new(points, 0, 0, None, None);
        let converted = path.to_integer_unit();

        assert_eq!(converted.width(), None);
    }

    #[test]
    fn test_path_with_different_unit_points() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::float(100.0, 100.0, 1e-6)];
        let path = Path::new(points, 0, 0, None, None);
        assert_eq!(path.points().len(), 2);
    }

    #[test]
    fn test_path_bounding_box() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 5, 1e-9),
            Point::integer(20, -3, 1e-9),
        ];
        let path = Path::new(points, 1, 0, None, None);
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
        let path = Path::new(points, 1, 0, None, None);
        let (min, max) = path.bounding_box();
        assert_eq!(min, Point::integer(5, 10, 1e-9));
        assert_eq!(max, Point::integer(5, 10, 1e-9));
    }

    #[allow(clippy::needless_pass_by_value)]
    mod property_tests {
        use super::*;
        use quickcheck_macros::quickcheck;

        #[quickcheck]
        fn bounding_box_contains_all_points(path: Path) -> bool {
            if path.points().is_empty() {
                return true;
            }
            let (min, max) = path.bounding_box();
            path.points().iter().all(|p| {
                p.x().float_value() >= min.x().float_value()
                    && p.x().float_value() <= max.x().float_value()
                    && p.y().float_value() >= min.y().float_value()
                    && p.y().float_value() <= max.y().float_value()
            })
        }

        #[quickcheck]
        fn translation_preserves_point_count(path: Path, dx: i32, dy: i32) -> bool {
            let units = path.points()[0].units().0;
            let dx = (dx % 10_000).clamp(-10_000, 10_000);
            let dy = (dy % 10_000).clamp(-10_000, 10_000);
            let delta = Point::integer(dx, dy, units);
            let translated = path.clone().translate(delta);
            translated.points().len() == path.points().len()
        }
    }
}
