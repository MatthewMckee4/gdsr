use crate::{
    DataType, Dimensions, Layer, LayerMapping, Movable, Point, Radians, Transformable, Unit,
};

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
    pub(crate) begin_extension: Option<Unit>,
    pub(crate) end_extension: Option<Unit>,
}

impl Path {
    /// Returns a builder for constructing a path with method chaining.
    pub fn builder() -> PathBuilder {
        PathBuilder::default()
    }

    /// Creates a new path from the given points, layer, data type, optional end cap type, optional width, and optional extensions.
    pub fn new(
        points: impl IntoIterator<Item = Point>,
        layer: Layer,
        data_type: DataType,
        path_type: Option<PathType>,
        width: Option<Unit>,
        begin_extension: Option<Unit>,
        end_extension: Option<Unit>,
    ) -> Self {
        Self {
            points: points.into_iter().collect(),
            layer,
            data_type,
            r#type: path_type,
            width,
            begin_extension,
            end_extension,
        }
    }

    /// Creates a curved arc path.
    ///
    /// Generates `num_points` along a circular arc from `start_angle` to `end_angle`,
    /// centered at `center` with the given `radius`. All points are equidistant from center.
    /// Supports arcs in both directions depending on angle order.
    pub fn arc(
        center: Point,
        radius: f64,
        start_angle: Radians,
        end_angle: Radians,
        num_points: usize,
        layer: Layer,
        data_type: DataType,
        width: Option<Unit>,
    ) -> Self {
        let num_points = num_points.max(2);
        let x_units = center.x().units();
        let y_units = center.y().units();

        let angle_span = end_angle.value() - start_angle.value();

        let points = (0..num_points).map(|i| {
            let t = if num_points > 1 {
                i as f64 / (num_points - 1) as f64
            } else {
                0.0
            };
            let angle = start_angle.value() + t * angle_span;
            center
                + Point::new(
                    Unit::float(radius * angle.cos(), x_units),
                    Unit::float(radius * angle.sin(), y_units),
                )
        });

        Self::new(points, layer, data_type, None, width, None, None)
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

    /// Remaps the layer and data type using the given mapping.
    /// If the current (layer, `data_type`) pair is found in the mapping, it is replaced.
    pub fn remap_layers(&mut self, mapping: &LayerMapping) {
        if let Some(&(new_layer, new_data_type)) = mapping.get(&(self.layer, self.data_type)) {
            self.layer = new_layer;
            self.data_type = new_data_type;
        }
    }

    /// Returns the end cap type, if set.
    pub const fn path_type(&self) -> &Option<PathType> {
        &self.r#type
    }

    /// Returns the path width, if set.
    pub const fn width(&self) -> Option<Unit> {
        self.width
    }

    /// Returns the beginning extension distance, if set.
    pub const fn begin_extension(&self) -> Option<Unit> {
        self.begin_extension
    }

    /// Returns the end extension distance, if set.
    pub const fn end_extension(&self) -> Option<Unit> {
        self.end_extension
    }

    /// Converts all points, width, and extensions to integer units.
    #[must_use]
    pub fn to_integer_unit(self) -> Self {
        Self {
            points: self.points.iter().map(Point::to_integer_unit).collect(),
            width: self.width.map(Unit::to_integer_unit),
            begin_extension: self.begin_extension.map(Unit::to_integer_unit),
            end_extension: self.end_extension.map(Unit::to_integer_unit),
            ..self
        }
    }

    /// Expands this path to the polygon vertices representing its filled area.
    ///
    /// Returns `None` if the path has no width or fewer than 2 points.
    /// `arc_segments` controls the number of segments used for round end caps.
    pub fn to_polygon_points(&self, arc_segments: usize) -> Option<Vec<Point>> {
        let width = self.width?;
        if self.points.len() < 2 {
            return None;
        }

        let half_width = width.absolute_value() / 2.0;
        if half_width <= 0.0 {
            return None;
        }

        let path_type = self.r#type.unwrap_or_default();
        let begin_ext = self.begin_extension.map_or(0.0, |u| u.absolute_value());
        let end_ext = self.end_extension.map_or(0.0, |u| u.absolute_value());

        let units = self.points[0].units().0;
        let centerline: Vec<(f64, f64)> = self
            .points
            .iter()
            .map(|p| (p.x().float_value(), p.y().float_value()))
            .collect();

        let half_width_scaled = half_width / units;
        let begin_ext_scaled = begin_ext / units;
        let end_ext_scaled = end_ext / units;

        let expanded = crate::geometry::expand_path_to_polygon(
            &centerline,
            half_width_scaled,
            path_type,
            begin_ext_scaled,
            end_ext_scaled,
            arc_segments,
        );

        if expanded.is_empty() {
            return None;
        }

        Some(
            expanded
                .into_iter()
                .map(|(x, y)| Point::float(x, y, units))
                .collect(),
        )
    }

    /// Converts all points, width, and extensions to float units.
    #[must_use]
    pub fn to_float_unit(self) -> Self {
        Self {
            points: self.points.iter().map(Point::to_float_unit).collect(),
            width: self.width.map(Unit::to_float_unit),
            begin_extension: self.begin_extension.map(Unit::to_float_unit),
            end_extension: self.end_extension.map(Unit::to_float_unit),
            ..self
        }
    }
}

/// A builder for constructing [`Path`] instances with method chaining.
///
/// All fields have sensible defaults (empty points, layer 0, data type 0,
/// all optional fields `None`).
///
/// # Example
/// ```
/// # use gdsr::{Path, PathType, Layer, DataType, Point, Unit};
/// let path = Path::builder()
///     .points(vec![
///         Point::integer(0, 0, 1e-9),
///         Point::integer(10, 0, 1e-9),
///     ])
///     .layer(Layer::new(5))
///     .width(Unit::default_integer(100))
///     .path_type(PathType::Round)
///     .build();
/// ```
#[derive(Clone, Debug, Default)]
pub struct PathBuilder {
    points: Vec<Point>,
    layer: Layer,
    data_type: DataType,
    path_type: Option<PathType>,
    width: Option<Unit>,
    begin_extension: Option<Unit>,
    end_extension: Option<Unit>,
}

impl PathBuilder {
    /// Sets the path's points.
    #[must_use]
    pub fn points(mut self, points: impl IntoIterator<Item = Point>) -> Self {
        self.points = points.into_iter().collect();
        self
    }

    /// Sets the path's layer.
    #[must_use]
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the path's data type.
    #[must_use]
    pub fn data_type(mut self, data_type: DataType) -> Self {
        self.data_type = data_type;
        self
    }

    /// Sets the path's end cap type.
    #[must_use]
    pub fn path_type(mut self, path_type: PathType) -> Self {
        self.path_type = Some(path_type);
        self
    }

    /// Sets the path's width.
    #[must_use]
    pub fn width(mut self, width: Unit) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the path's begin extension distance.
    #[must_use]
    pub fn begin_extension(mut self, ext: Unit) -> Self {
        self.begin_extension = Some(ext);
        self
    }

    /// Sets the path's end extension distance.
    #[must_use]
    pub fn end_extension(mut self, ext: Unit) -> Self {
        self.end_extension = Some(ext);
        self
    }

    /// Builds the path.
    pub fn build(self) -> Path {
        Path::new(
            self.points,
            self.layer,
            self.data_type,
            self.path_type,
            self.width,
            self.begin_extension,
            self.end_extension,
        )
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
        )?;
        if let Some(begin_ext) = self.begin_extension() {
            write!(f, ", begin_extension {begin_ext}")?;
        }
        if let Some(end_ext) = self.end_extension() {
            write!(f, ", end_extension {end_ext}")?;
        }
        Ok(())
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
            Some(Unit::default_integer(5)),
            Some(Unit::default_integer(15)),
        );

        assert_eq!(path.points(), &points);
        assert_eq!(path.layer(), Layer::new(1));
        assert_eq!(path.data_type(), DataType::new(2));
        assert_eq!(path.path_type(), &Some(PathType::Round));
        assert_eq!(path.width(), Some(Unit::default_integer(10)));
        assert_eq!(path.begin_extension(), Some(Unit::default_integer(5)));
        assert_eq!(path.end_extension(), Some(Unit::default_integer(15)));
    }

    #[test]
    fn test_path_default() {
        let path = Path::default();

        assert!(path.points().is_empty());
        assert_eq!(path.layer(), Layer::new(0));
        assert_eq!(path.data_type(), DataType::new(0));
        assert_eq!(path.path_type(), &None);
        assert_eq!(path.width(), None);
        assert_eq!(path.begin_extension(), None);
        assert_eq!(path.end_extension(), None);
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
            None,
            None,
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
            None,
            None,
        );
        let path2 = path1.clone();

        assert_eq!(path1, path2);

        let path3 = Path::new(
            points,
            Layer::new(1),
            DataType::new(2),
            Some(PathType::Square),
            Some(Unit::default_integer(5)),
            None,
            None,
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
            None,
            None,
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
            None,
            None,
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
        let path = Path::new(
            points,
            Layer::new(0),
            DataType::new(0),
            None,
            None,
            None,
            None,
        );
        let converted = path.to_integer_unit();

        assert_eq!(converted.width(), None);
    }

    #[test]
    fn test_path_with_different_unit_points() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::float(100.0, 100.0, 1e-6)];
        let path = Path::new(
            points,
            Layer::new(0),
            DataType::new(0),
            None,
            None,
            None,
            None,
        );
        assert_eq!(path.points().len(), 2);
    }

    #[test]
    fn test_path_bounding_box() {
        let points = vec![
            Point::integer(0, 0, 1e-9),
            Point::integer(10, 5, 1e-9),
            Point::integer(20, -3, 1e-9),
        ];
        let path = Path::new(
            points,
            Layer::new(1),
            DataType::new(0),
            None,
            None,
            None,
            None,
        );
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
        let path = Path::new(
            points,
            Layer::new(1),
            DataType::new(0),
            None,
            None,
            None,
            None,
        );
        let (min, max) = path.bounding_box();
        assert_eq!(min, Point::integer(5, 10, 1e-9));
        assert_eq!(max, Point::integer(5, 10, 1e-9));
    }

    #[test]
    fn test_arc_point_count() {
        let center = Point::float(0.0, 0.0, 1e-6);
        let arc = Path::arc(
            center,
            10.0,
            Radians::new(0.0),
            Radians::PI,
            20,
            Layer::new(0),
            DataType::new(0),
            None,
        );
        assert_eq!(arc.points().len(), 20);
    }

    #[test]
    fn test_arc_equidistant_from_center() {
        let center = Point::float(5.0, 5.0, 1e-6);
        let radius = 10.0;
        let arc = Path::arc(
            center,
            radius,
            Radians::new(0.0),
            Radians::TAU,
            36,
            Layer::new(0),
            DataType::new(0),
            None,
        );
        let tolerance = 1e-9;
        for point in arc.points() {
            let dx = point.x().float_value() - center.x().float_value();
            let dy = point.y().float_value() - center.y().float_value();
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(
                (dist - radius).abs() < tolerance,
                "point distance {dist} differs from radius {radius}"
            );
        }
    }

    #[test]
    fn test_arc_quarter_circle() {
        let center = Point::float(0.0, 0.0, 1e-6);
        let radius = 10.0;
        let arc = Path::arc(
            center,
            radius,
            Radians::new(0.0),
            Radians::FRAC_PI_2,
            5,
            Layer::new(1),
            DataType::new(2),
            Some(Unit::default_float(1.0)),
        );
        let tolerance = 1e-9;
        let first = arc.points().first().expect("arc should have points");
        assert!((first.x().float_value() - radius).abs() < tolerance);
        assert!(first.y().float_value().abs() < tolerance);

        let last = arc.points().last().expect("arc should have points");
        assert!(last.x().float_value().abs() < tolerance);
        assert!((last.y().float_value() - radius).abs() < tolerance);
    }

    #[test]
    fn test_builder_minimal() {
        let path = Path::builder()
            .points(vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
            ])
            .build();

        assert_eq!(path.points().len(), 2);
        assert_eq!(path.layer(), Layer::new(0));
        assert_eq!(path.data_type(), DataType::new(0));
        assert_eq!(path.width(), None);
        assert_eq!(path.path_type(), &None);
        assert_eq!(path.begin_extension(), None);
        assert_eq!(path.end_extension(), None);
    }

    #[test]
    fn test_builder_with_all_fields() {
        let path = Path::builder()
            .points(vec![
                Point::integer(0, 0, 1e-9),
                Point::integer(10, 0, 1e-9),
            ])
            .layer(Layer::new(5))
            .data_type(DataType::new(3))
            .width(Unit::default_integer(100))
            .path_type(PathType::Round)
            .begin_extension(Unit::default_integer(5))
            .end_extension(Unit::default_integer(15))
            .build();

        assert_eq!(path.layer(), Layer::new(5));
        assert_eq!(path.data_type(), DataType::new(3));
        assert_eq!(path.width(), Some(Unit::default_integer(100)));
        assert_eq!(path.path_type(), &Some(PathType::Round));
        assert_eq!(path.begin_extension(), Some(Unit::default_integer(5)));
        assert_eq!(path.end_extension(), Some(Unit::default_integer(15)));
    }

    #[test]
    fn test_builder_defaults() {
        let path = Path::builder().build();

        assert!(path.points().is_empty());
        assert_eq!(path.layer(), Layer::new(0));
        assert_eq!(path.data_type(), DataType::new(0));
        assert_eq!(path.width(), None);
    }

    #[test]
    fn test_builder_matches_new() {
        let points = vec![Point::integer(0, 0, 1e-9), Point::integer(10, 0, 1e-9)];
        let from_new = Path::new(
            points.clone(),
            Layer::new(2),
            DataType::new(1),
            Some(PathType::Overlap),
            Some(Unit::default_integer(50)),
            Some(Unit::default_integer(10)),
            Some(Unit::default_integer(20)),
        );
        let from_builder = Path::builder()
            .points(points)
            .layer(Layer::new(2))
            .data_type(DataType::new(1))
            .path_type(PathType::Overlap)
            .width(Unit::default_integer(50))
            .begin_extension(Unit::default_integer(10))
            .end_extension(Unit::default_integer(20))
            .build();

        assert_eq!(from_new, from_builder);
    }

    #[test]
    fn test_arc_num_points_clamped_to_two() {
        let center = Point::float(0.0, 0.0, 1e-6);
        let arc = Path::arc(
            center,
            5.0,
            Radians::new(0.0),
            Radians::PI,
            1,
            Layer::new(0),
            DataType::new(0),
            None,
        );
        assert_eq!(arc.points().len(), 2);
    }
}
