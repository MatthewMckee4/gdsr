use geo::{
    BooleanOps, Coord, LineString, MultiLineString, Polygon as GeoPolygon, TriangulateEarcut,
};

use crate::{Cell, Element, GdsBox, Library, Node, Path, Point, Polygon, Unit};

const MIN_POLYGON_POINTS: usize = 4;

struct ClipRegion {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    polygon: GeoPolygon<f64>,
}

enum BoundsRelation {
    Outside,
    Inside,
    Intersecting,
}

impl ClipRegion {
    fn new((corner_1, corner_2): (Point, Point)) -> Self {
        let min_x = corner_1
            .x()
            .absolute_value()
            .min(corner_2.x().absolute_value());
        let min_y = corner_1
            .y()
            .absolute_value()
            .min(corner_2.y().absolute_value());
        let max_x = corner_1
            .x()
            .absolute_value()
            .max(corner_2.x().absolute_value());
        let max_y = corner_1
            .y()
            .absolute_value()
            .max(corner_2.y().absolute_value());
        let polygon = GeoPolygon::new(
            LineString::new(vec![
                Coord { x: min_x, y: min_y },
                Coord { x: max_x, y: min_y },
                Coord { x: max_x, y: max_y },
                Coord { x: min_x, y: max_y },
                Coord { x: min_x, y: min_y },
            ]),
            Vec::new(),
        );

        Self {
            min_x,
            min_y,
            max_x,
            max_y,
            polygon,
        }
    }

    fn contains(&self, point: &Point) -> bool {
        let x = point.x().absolute_value();
        let y = point.y().absolute_value();
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    fn relation(&self, points: &[Point]) -> BoundsRelation {
        let Some(first) = points.first() else {
            return BoundsRelation::Outside;
        };
        let first = coordinate(first);
        let mut min_x = first.x;
        let mut min_y = first.y;
        let mut max_x = first.x;
        let mut max_y = first.y;

        for point in &points[1..] {
            let point = coordinate(point);
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        if max_x < self.min_x || min_x > self.max_x || max_y < self.min_y || min_y > self.max_y {
            BoundsRelation::Outside
        } else if min_x >= self.min_x
            && max_x <= self.max_x
            && min_y >= self.min_y
            && max_y <= self.max_y
        {
            BoundsRelation::Inside
        } else {
            BoundsRelation::Intersecting
        }
    }
}

#[derive(Clone, Copy)]
struct PointUnits {
    x: f64,
    y: f64,
}

impl PointUnits {
    fn from_point(point: &Point) -> Self {
        let (x, y) = point.units();
        Self { x, y }
    }

    fn point(self, coordinate: Coord<f64>) -> Point {
        Point::new(
            Unit::float(coordinate.x / self.x, self.x),
            Unit::float(coordinate.y / self.y, self.y),
        )
    }
}

fn coordinate(point: &Point) -> Coord<f64> {
    Coord {
        x: point.x().absolute_value(),
        y: point.y().absolute_value(),
    }
}

fn closed_line_string(points: &[Point]) -> Option<LineString<f64>> {
    let mut coordinates: Vec<Coord<f64>> = points.iter().map(coordinate).collect();
    coordinates.dedup();
    if coordinates.first() != coordinates.last()
        && let Some(first) = coordinates.first().copied()
    {
        coordinates.push(first);
    }
    (coordinates.len() >= MIN_POLYGON_POINTS).then(|| LineString::new(coordinates))
}

pub fn polygon_to_geo(polygon: &Polygon) -> Option<GeoPolygon<f64>> {
    closed_line_string(polygon.points()).map(|exterior| GeoPolygon::new(exterior, Vec::new()))
}

fn polygon_from_coordinates(
    coordinates: impl IntoIterator<Item = Coord<f64>>,
    units: PointUnits,
    source: &Polygon,
) -> Option<Polygon> {
    let mut points: Vec<Point> = coordinates
        .into_iter()
        .map(|coordinate| units.point(coordinate))
        .collect();
    points.dedup();
    if points.first() != points.last()
        && let Some(first) = points.first().copied()
    {
        points.push(first);
    }
    (points.len() >= MIN_POLYGON_POINTS).then(|| {
        let mut polygon = Polygon::new(points, source.layer(), source.data_type());
        polygon
            .properties_mut()
            .extend_from_slice(source.properties());
        polygon
    })
}

pub fn polygons_from_geo(polygon: &GeoPolygon<f64>, source: &Polygon) -> Vec<Polygon> {
    let Some(first_point) = source.points().first() else {
        return Vec::new();
    };
    let units = PointUnits::from_point(first_point);

    if polygon.interiors().is_empty() {
        return polygon_from_coordinates(polygon.exterior().0.iter().copied(), units, source)
            .into_iter()
            .collect();
    }

    polygon
        .earcut_triangles()
        .into_iter()
        .filter_map(|triangle| {
            let coordinates = [triangle.v1(), triangle.v2(), triangle.v3()];
            let twice_area = (coordinates[1].x - coordinates[0].x)
                * (coordinates[2].y - coordinates[0].y)
                - (coordinates[1].y - coordinates[0].y) * (coordinates[2].x - coordinates[0].x);
            (twice_area != 0.0)
                .then(|| polygon_from_coordinates(coordinates, units, source))
                .flatten()
        })
        .collect()
}

fn clip_polygon(polygon: &Polygon, region: &ClipRegion) -> Vec<Polygon> {
    if polygon.points().len() < MIN_POLYGON_POINTS {
        return Vec::new();
    }
    match region.relation(polygon.points()) {
        BoundsRelation::Outside => return Vec::new(),
        BoundsRelation::Inside => return vec![polygon.clone()],
        BoundsRelation::Intersecting => {}
    }
    let Some(polygon_geometry) = polygon_to_geo(polygon) else {
        return Vec::new();
    };

    polygon_geometry
        .intersection(&region.polygon)
        .0
        .into_iter()
        .flat_map(|clipped_polygon| polygons_from_geo(&clipped_polygon, polygon))
        .collect()
}

fn clip_path(path: &Path, region: &ClipRegion) -> Vec<Path> {
    let Some(first_point) = path.points().first() else {
        return Vec::new();
    };
    match region.relation(path.points()) {
        BoundsRelation::Outside => return Vec::new(),
        BoundsRelation::Inside => return vec![path.clone()],
        BoundsRelation::Intersecting => {}
    }
    if path.points().len() < 2 {
        return Vec::new();
    }

    let line = LineString::new(path.points().iter().map(coordinate).collect());
    let clipped = region
        .polygon
        .clip(&MultiLineString::new(vec![line]), false);
    let units = PointUnits::from_point(first_point);

    clipped
        .0
        .into_iter()
        .filter_map(|line| {
            let mut points: Vec<Point> = line
                .0
                .into_iter()
                .map(|coordinate| units.point(coordinate))
                .collect();
            points.dedup();
            (points.len() >= 2).then(|| {
                let mut clipped_path = Path::new(
                    points,
                    path.layer(),
                    path.data_type(),
                    *path.path_type(),
                    path.width(),
                    None,
                    None,
                );
                clipped_path
                    .properties_mut()
                    .extend_from_slice(path.properties());
                clipped_path
            })
        })
        .collect()
}

fn clip_box(gds_box: &GdsBox, region: &ClipRegion) -> Option<GdsBox> {
    if region.contains(&gds_box.bottom_left()) && region.contains(&gds_box.top_right()) {
        return Some(gds_box.clone());
    }

    let min_x = gds_box.bottom_left().x().absolute_value().max(region.min_x);
    let min_y = gds_box.bottom_left().y().absolute_value().max(region.min_y);
    let max_x = gds_box.top_right().x().absolute_value().min(region.max_x);
    let max_y = gds_box.top_right().y().absolute_value().min(region.max_y);
    if min_x >= max_x || min_y >= max_y {
        return None;
    }

    let units = PointUnits::from_point(&gds_box.bottom_left());
    let mut clipped_box = GdsBox::new(
        units.point(Coord { x: min_x, y: min_y }),
        units.point(Coord { x: max_x, y: max_y }),
        gds_box.layer(),
        gds_box.box_type(),
    );
    clipped_box
        .properties_mut()
        .extend_from_slice(gds_box.properties());
    Some(clipped_box)
}

fn clip_node(node: &Node, region: &ClipRegion) -> Option<Node> {
    let points: Vec<Point> = node
        .points()
        .iter()
        .copied()
        .filter(|point| region.contains(point))
        .collect();
    if points.is_empty() {
        return None;
    }
    if points.len() == node.points().len() {
        return Some(node.clone());
    }
    let mut clipped_node = Node::new(points, node.layer(), node.node_type());
    clipped_node
        .properties_mut()
        .extend_from_slice(node.properties());
    Some(clipped_node)
}

fn clip_element(element: &Element, region: &ClipRegion) -> Vec<Element> {
    match element {
        Element::Path(path) => clip_path(path, region)
            .into_iter()
            .map(Element::Path)
            .collect(),
        Element::Polygon(polygon) => clip_polygon(polygon, region)
            .into_iter()
            .map(Element::Polygon)
            .collect(),
        Element::Box(gds_box) => clip_box(gds_box, region)
            .map(Element::Box)
            .into_iter()
            .collect(),
        Element::Node(node) => clip_node(node, region)
            .map(Element::Node)
            .into_iter()
            .collect(),
        Element::Text(text) if region.contains(text.origin()) => vec![Element::Text(text.clone())],
        Element::Text(_) => Vec::new(),
        Element::Reference(reference) => vec![Element::Reference(reference.clone())],
    }
}

fn clip_cell(cell: &Cell, region: &ClipRegion) -> Cell {
    let mut clipped = Cell::new(cell.name());
    for element in cell.iter_elements() {
        for clipped_element in clip_element(element, region) {
            clipped.add(clipped_element);
        }
    }
    clipped
}

impl Polygon {
    /// Clips this polygon to an axis-aligned bounding box.
    ///
    /// Multiple polygons are returned when clipping splits a concave polygon into disconnected
    /// regions. Element properties are preserved. Bounding-box corners may be supplied in either
    /// order or in different units.
    #[must_use]
    pub fn clip_to_bounding_box(&self, bounds: (Point, Point)) -> Vec<Self> {
        clip_polygon(self, &ClipRegion::new(bounds))
    }
}

impl Path {
    /// Clips this path's centerline to an axis-aligned bounding box.
    ///
    /// A path can split into multiple paths. Partially clipped paths retain their width and path
    /// type and properties, but their end extensions are removed because the new endpoints lie on
    /// the boundary.
    #[must_use]
    pub fn clip_to_bounding_box(&self, bounds: (Point, Point)) -> Vec<Self> {
        clip_path(self, &ClipRegion::new(bounds))
    }
}

impl Element {
    /// Clips this element to an axis-aligned bounding box.
    ///
    /// Element properties are preserved. References are preserved because resolving them requires
    /// a library and a chosen hierarchy coordinate system.
    #[must_use]
    pub fn clip_to_bounding_box(&self, bounds: (Point, Point)) -> Vec<Self> {
        clip_element(self, &ClipRegion::new(bounds))
    }
}

impl Cell {
    /// Returns a cell whose direct geometry is clipped to an axis-aligned bounding box.
    ///
    /// References are preserved. Flatten the cell first when the referenced geometry must be
    /// clipped in this cell's coordinate system.
    #[must_use]
    pub fn clip_to_bounding_box(&self, bounds: (Point, Point)) -> Self {
        clip_cell(self, &ClipRegion::new(bounds))
    }
}

impl Library {
    /// Returns a library with each cell's direct geometry clipped in that cell's local coordinates.
    ///
    /// Cell names and references are preserved, including cells that become empty.
    #[must_use]
    pub fn clip_to_bounding_box(&self, bounds: (Point, Point)) -> Self {
        let region = ClipRegion::new(bounds);
        let mut clipped = Self::new(self.name());
        for cell in self.cells().values() {
            clipped.add_cell(clip_cell(cell, &region));
        }
        clipped
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::{DataType, Dimensions, Layer, PathType, Property, Reference, Text};

    const UNITS: f64 = 1e-9;

    fn point(x: i32, y: i32) -> Point {
        Point::integer(x, y, UNITS)
    }

    fn bounds() -> (Point, Point) {
        (point(0, 0), point(10, 10))
    }

    fn total_area(polygons: &[Polygon]) -> f64 {
        polygons
            .iter()
            .map(|polygon| polygon.area().float_value())
            .sum()
    }

    fn assert_points_in_bounds(points: &[Point], region: &ClipRegion) {
        assert!(points.iter().all(|point| region.contains(point)));
    }

    #[test]
    fn polygon_clipping_preserves_metadata_and_removes_outside_geometry() {
        let mut polygon =
            Polygon::rectangle(point(-5, -5), point(5, 5), Layer::new(3), DataType::new(4));
        polygon.properties_mut().push(Property::new(7, "source"));

        let clipped = polygon.clip_to_bounding_box(bounds());

        assert_eq!(clipped.len(), 1);
        assert_eq!(clipped[0].layer(), Layer::new(3));
        assert_eq!(clipped[0].data_type(), DataType::new(4));
        assert_eq!(clipped[0].properties(), polygon.properties());
        assert_relative_eq!(clipped[0].area().float_value(), 25.0, epsilon = 1e-6);
        assert_points_in_bounds(clipped[0].points(), &ClipRegion::new(bounds()));

        let outside = Polygon::rectangle(
            point(20, 20),
            point(30, 30),
            Layer::new(1),
            DataType::new(0),
        );
        assert!(outside.clip_to_bounding_box(bounds()).is_empty());

        let inside = Polygon::rectangle(point(1, 1), point(2, 2), Layer::new(1), DataType::new(0));
        assert_eq!(inside.clip_to_bounding_box(bounds()), vec![inside]);
    }

    #[test]
    fn concave_polygon_can_clip_into_disconnected_polygons() {
        let polygon = Polygon::new(
            [
                point(0, 0),
                point(10, 0),
                point(10, 10),
                point(7, 10),
                point(7, 3),
                point(3, 3),
                point(3, 10),
                point(0, 10),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let clipping_bounds = (point(0, 5), point(10, 10));

        let clipped = polygon.clip_to_bounding_box(clipping_bounds);

        assert_eq!(clipped.len(), 2);
        assert_relative_eq!(total_area(&clipped), 30.0, epsilon = 1e-6);
        let region = ClipRegion::new(clipping_bounds);
        assert!(
            clipped
                .iter()
                .all(|polygon| polygon.points().iter().all(|point| region.contains(point)))
        );
    }

    #[test]
    fn polygon_clipping_supports_mixed_units_and_reversed_bounds() {
        let polygon = Polygon::rectangle(
            point(-500, -500),
            point(1_500, 1_500),
            Layer::new(1),
            DataType::new(0),
        );
        let clipping_bounds = (Point::float(1.0, 1.0, 1e-6), Point::float(0.0, 0.0, 1e-6));

        let clipped = polygon.clip_to_bounding_box(clipping_bounds);

        assert_eq!(clipped.len(), 1);
        assert_eq!(
            clipped[0].bounding_box(),
            (point(0, 0), point(1_000, 1_000))
        );
    }

    #[test]
    fn path_clipping_splits_reentry_and_removes_new_end_extensions() {
        let path = Path::new(
            [
                point(-5, 2),
                point(5, 2),
                point(15, 2),
                point(15, 8),
                point(5, 8),
                point(-5, 8),
            ],
            Layer::new(4),
            DataType::new(5),
            Some(PathType::Round),
            Some(Unit::integer(2, UNITS)),
            Some(Unit::integer(1, UNITS)),
            Some(Unit::integer(1, UNITS)),
        );

        let clipped = path.clip_to_bounding_box(bounds());

        assert_eq!(clipped.len(), 2);
        let region = ClipRegion::new(bounds());
        for clipped_path in clipped {
            assert_eq!(clipped_path.layer(), Layer::new(4));
            assert_eq!(clipped_path.data_type(), DataType::new(5));
            assert_eq!(*clipped_path.path_type(), Some(PathType::Round));
            assert_eq!(clipped_path.width(), Some(Unit::integer(2, UNITS)));
            assert_eq!(clipped_path.begin_extension(), None);
            assert_eq!(clipped_path.end_extension(), None);
            assert_points_in_bounds(clipped_path.points(), &region);
        }

        let inside = Path::new(
            [point(1, 1), point(2, 2)],
            Layer::new(1),
            DataType::new(0),
            None,
            None,
            Some(Unit::integer(1, UNITS)),
            Some(Unit::integer(1, UNITS)),
        );
        assert_eq!(inside.clip_to_bounding_box(bounds()), vec![inside]);
    }

    #[test]
    fn polygon_holes_are_decomposed_without_filling_them() {
        let source =
            Polygon::rectangle(point(0, 0), point(10, 10), Layer::new(6), DataType::new(7));
        let scale = |value: f64| value * UNITS;
        let ring = |coordinates: &[(f64, f64)]| {
            LineString::new(
                coordinates
                    .iter()
                    .map(|&(x, y)| Coord {
                        x: scale(x),
                        y: scale(y),
                    })
                    .collect(),
            )
        };
        let polygon = GeoPolygon::new(
            ring(&[
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ]),
            vec![ring(&[
                (3.0, 3.0),
                (3.0, 7.0),
                (7.0, 7.0),
                (7.0, 3.0),
                (3.0, 3.0),
            ])],
        );

        let decomposed = polygons_from_geo(&polygon, &source);

        assert!(decomposed.len() > 1);
        assert_relative_eq!(total_area(&decomposed), 84.0, epsilon = 1e-9);
        assert!(decomposed.iter().all(|polygon| {
            polygon.layer() == Layer::new(6) && polygon.data_type() == DataType::new(7)
        }));
    }

    #[test]
    fn cell_clipping_handles_concrete_elements_and_preserves_references() {
        let mut cell = Cell::new("top");
        cell.add(Polygon::rectangle(
            point(20, 20),
            point(30, 30),
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(GdsBox::new(
            point(-5, 2),
            point(5, 8),
            Layer::new(2),
            DataType::new(3),
        ));
        cell.add(Node::new(
            vec![point(-1, 1), point(1, 1)],
            Layer::new(4),
            DataType::new(5),
        ));
        cell.add(Text::default().set_origin(point(2, 2)));
        cell.add(Text::default().set_origin(point(20, 20)));
        cell.add(Reference::new("child"));

        let clipped = cell.clip_to_bounding_box(bounds());

        assert_eq!(cell.elements().len(), 6);
        assert_eq!(clipped.name(), "top");
        assert_eq!(clipped.elements().len(), 4);
        assert_eq!(clipped.polygons().count(), 0);
        let Some(gds_box) = clipped.boxes().next() else {
            panic!("clipped box should remain");
        };
        assert_eq!(gds_box.bottom_left(), point(0, 2));
        assert_eq!(gds_box.top_right(), point(5, 8));
        let Some(node) = clipped.nodes().next() else {
            panic!("partially contained node should remain");
        };
        assert_eq!(node.points(), &[point(1, 1)]);
        assert_eq!(clipped.texts().count(), 1);
        assert_eq!(clipped.references().count(), 1);
    }

    #[test]
    fn library_clipping_is_non_mutating_and_preserves_empty_cells() {
        let polygon =
            Polygon::rectangle(point(-5, -5), point(5, 5), Layer::new(1), DataType::new(0));
        let mut source_cell = Cell::new("source");
        source_cell.add(polygon);
        let empty_cell = Cell::new("empty");
        let mut library = Library::new("library");
        library.add_cell(source_cell);
        library.add_cell(empty_cell);

        let clipped = library.clip_to_bounding_box(bounds());

        assert_eq!(library.name(), "library");
        assert_eq!(library.cells().len(), 2);
        assert_eq!(clipped.name(), "library");
        assert_eq!(clipped.cells().len(), 2);
        let Some(original) = library.get_cell("source") else {
            panic!("source cell should exist");
        };
        let Some(clipped_source) = clipped.get_cell("source") else {
            panic!("clipped source cell should exist");
        };
        assert_relative_eq!(
            original
                .polygons()
                .next()
                .map_or(0.0, |p| p.area().float_value()),
            100.0
        );
        assert_relative_eq!(
            clipped_source
                .polygons()
                .next()
                .map_or(0.0, |p| p.area().float_value()),
            25.0,
            epsilon = 1e-6
        );
        assert!(
            clipped
                .get_cell("empty")
                .is_some_and(|cell| cell.elements().is_empty())
        );
    }
}
