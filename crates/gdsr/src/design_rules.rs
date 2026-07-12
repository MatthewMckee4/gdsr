use crate::{Cell, DataType, Dimensions, Element, Layer, Library, Point, Unit};

const EPSILON: f64 = 1e-12;

/// Options for basic design-rule validation.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignRuleOptions {
    pub min_path_width: Option<Unit>,
    pub min_spacing: Option<Unit>,
    pub check_self_intersections: bool,
    pub check_zero_area_polygons: bool,
    pub check_zero_length_paths: bool,
}

impl Default for DesignRuleOptions {
    fn default() -> Self {
        Self {
            min_path_width: None,
            min_spacing: None,
            check_self_intersections: true,
            check_zero_area_polygons: true,
            check_zero_length_paths: true,
        }
    }
}

impl DesignRuleOptions {
    pub fn basic() -> Self {
        Self::default()
    }
}

/// Structured design-rule validation result.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DesignRuleReport {
    pub violations: Vec<DesignRuleViolation>,
}

impl DesignRuleReport {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// A single design-rule violation.
#[derive(Clone, Debug, PartialEq)]
pub struct DesignRuleViolation {
    pub cell_name: String,
    pub element_index: usize,
    pub other_element_index: Option<usize>,
    pub kind: DesignRuleViolationKind,
    pub location: Option<Point>,
    pub message: String,
}

/// Kinds of design-rule violations reported by the basic validator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesignRuleViolationKind {
    SelfIntersectingPolygon,
    ZeroAreaPolygon,
    ZeroLengthPath,
    PathWidthBelowMinimum,
    SameLayerSpacingBelowMinimum,
}

pub fn validate_library(library: &Library, options: &DesignRuleOptions) -> DesignRuleReport {
    let mut report = DesignRuleReport::default();
    for cell in library.cells().values() {
        validate_cell(cell, options, &mut report);
    }
    report
}

fn validate_cell(cell: &Cell, options: &DesignRuleOptions, report: &mut DesignRuleReport) {
    for (index, element) in cell.elements().iter().enumerate() {
        match element {
            Element::Polygon(polygon) => {
                if options.check_zero_area_polygons
                    && polygon.area().absolute_value().abs() <= EPSILON
                {
                    report.violations.push(violation(
                        cell.name(),
                        index,
                        None,
                        DesignRuleViolationKind::ZeroAreaPolygon,
                        polygon.points().first().copied(),
                        "Polygon has zero area",
                    ));
                }
                if options.check_self_intersections && polygon_self_intersects(polygon.points()) {
                    report.violations.push(violation(
                        cell.name(),
                        index,
                        None,
                        DesignRuleViolationKind::SelfIntersectingPolygon,
                        polygon.points().first().copied(),
                        "Polygon self-intersects",
                    ));
                }
            }
            Element::Path(path) => {
                if options.check_zero_length_paths && path_length(path.points()) <= EPSILON {
                    report.violations.push(violation(
                        cell.name(),
                        index,
                        None,
                        DesignRuleViolationKind::ZeroLengthPath,
                        path.points().first().copied(),
                        "Path has zero length",
                    ));
                }
                if let (Some(width), Some(min_width)) = (path.width(), options.min_path_width) {
                    if width.absolute_value() < min_width.absolute_value() {
                        report.violations.push(violation(
                            cell.name(),
                            index,
                            None,
                            DesignRuleViolationKind::PathWidthBelowMinimum,
                            path.points().first().copied(),
                            "Path width is below the configured minimum",
                        ));
                    }
                }
            }
            Element::Box(_) | Element::Node(_) | Element::Text(_) | Element::Reference(_) => {}
        }
    }

    if let Some(min_spacing) = options.min_spacing {
        validate_spacing(cell, min_spacing.absolute_value(), report);
    }
}

fn validate_spacing(cell: &Cell, min_spacing: f64, report: &mut DesignRuleReport) {
    let indexed: Vec<(usize, Layer, DataType, RealBBox)> = cell
        .elements()
        .iter()
        .enumerate()
        .filter_map(|(index, element)| {
            let (layer, data_type) = element_layer_key(element)?;
            Some((index, layer, data_type, bbox(element)))
        })
        .collect();

    for (i, (left_index, left_layer, _, left_bbox)) in indexed.iter().enumerate() {
        for (right_index, right_layer, _, right_bbox) in indexed.iter().skip(i + 1) {
            if left_layer != right_layer {
                continue;
            }
            let spacing = bbox_distance(*left_bbox, *right_bbox);
            if spacing < min_spacing {
                report.violations.push(DesignRuleViolation {
                    cell_name: cell.name().to_string(),
                    element_index: *left_index,
                    other_element_index: Some(*right_index),
                    kind: DesignRuleViolationKind::SameLayerSpacingBelowMinimum,
                    location: Some(midpoint(left_bbox.center(), right_bbox.center())),
                    message: "Same-layer element spacing is below the configured minimum"
                        .to_string(),
                });
            }
        }
    }
}

fn violation(
    cell_name: &str,
    element_index: usize,
    other_element_index: Option<usize>,
    kind: DesignRuleViolationKind,
    location: Option<Point>,
    message: &str,
) -> DesignRuleViolation {
    DesignRuleViolation {
        cell_name: cell_name.to_string(),
        element_index,
        other_element_index,
        kind,
        location,
        message: message.to_string(),
    }
}

fn element_layer_key(element: &Element) -> Option<(Layer, DataType)> {
    match element {
        Element::Path(path) => Some((path.layer(), path.data_type())),
        Element::Polygon(polygon) => Some((polygon.layer(), polygon.data_type())),
        Element::Box(gds_box) => Some((gds_box.layer(), gds_box.box_type())),
        Element::Node(node) => Some((node.layer(), node.node_type())),
        Element::Text(text) => Some((text.layer(), text.data_type())),
        Element::Reference(_) => None,
    }
}

fn path_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| distance(pair[0], pair[1]))
        .sum()
}

fn distance(a: Point, b: Point) -> f64 {
    let ax = a.x().absolute_value();
    let ay = a.y().absolute_value();
    let bx = b.x().absolute_value();
    let by = b.y().absolute_value();
    (ax - bx).hypot(ay - by)
}

fn polygon_self_intersects(points: &[Point]) -> bool {
    if points.len() < 4 {
        return false;
    }

    let segments: Vec<(Point, Point)> = points.windows(2).map(|pair| (pair[0], pair[1])).collect();
    for (i, &(a, b)) in segments.iter().enumerate() {
        for (j, &(c, d)) in segments.iter().enumerate().skip(i + 1) {
            if segments_are_adjacent(i, j, segments.len()) {
                continue;
            }
            if segments_intersect(a, b, c, d) {
                return true;
            }
        }
    }
    false
}

fn segments_are_adjacent(left: usize, right: usize, segment_count: usize) -> bool {
    left.abs_diff(right) <= 1 || (left == 0 && right + 1 == segment_count)
}

fn segments_intersect(a: Point, b: Point, c: Point, d: Point) -> bool {
    let a = real_point(a);
    let b = real_point(b);
    let c = real_point(c);
    let d = real_point(d);

    let o1 = orientation(a, b, c);
    let o2 = orientation(a, b, d);
    let o3 = orientation(c, d, a);
    let o4 = orientation(c, d, b);

    if o1.abs() <= EPSILON && on_segment(a, c, b) {
        return true;
    }
    if o2.abs() <= EPSILON && on_segment(a, d, b) {
        return true;
    }
    if o3.abs() <= EPSILON && on_segment(c, a, d) {
        return true;
    }
    if o4.abs() <= EPSILON && on_segment(c, b, d) {
        return true;
    }

    (o1 > 0.0) != (o2 > 0.0) && (o3 > 0.0) != (o4 > 0.0)
}

fn orientation(a: RealPoint, b: RealPoint, c: RealPoint) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn on_segment(a: RealPoint, b: RealPoint, c: RealPoint) -> bool {
    b.x >= a.x.min(c.x) - EPSILON
        && b.x <= a.x.max(c.x) + EPSILON
        && b.y >= a.y.min(c.y) - EPSILON
        && b.y <= a.y.max(c.y) + EPSILON
}

#[derive(Clone, Copy)]
struct RealPoint {
    x: f64,
    y: f64,
}

fn real_point(point: Point) -> RealPoint {
    RealPoint {
        x: point.x().absolute_value(),
        y: point.y().absolute_value(),
    }
}

#[derive(Clone, Copy)]
struct RealBBox {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl RealBBox {
    fn center(self) -> RealPoint {
        RealPoint {
            x: f64::midpoint(self.min_x, self.max_x),
            y: f64::midpoint(self.min_y, self.max_y),
        }
    }
}

fn bbox(element: &Element) -> RealBBox {
    let (min, max) = element.bounding_box();
    RealBBox {
        min_x: min.x().absolute_value().min(max.x().absolute_value()),
        min_y: min.y().absolute_value().min(max.y().absolute_value()),
        max_x: min.x().absolute_value().max(max.x().absolute_value()),
        max_y: min.y().absolute_value().max(max.y().absolute_value()),
    }
}

fn bbox_distance(left: RealBBox, right: RealBBox) -> f64 {
    let dx = if left.max_x < right.min_x {
        right.min_x - left.max_x
    } else if right.max_x < left.min_x {
        left.min_x - right.max_x
    } else {
        0.0
    };
    let dy = if left.max_y < right.min_y {
        right.min_y - left.max_y
    } else if right.max_y < left.min_y {
        left.min_y - right.max_y
    } else {
        0.0
    };
    dx.hypot(dy)
}

fn midpoint(left: RealPoint, right: RealPoint) -> Point {
    Point::float(
        f64::midpoint(left.x, right.x),
        f64::midpoint(left.y, right.y),
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Path, PathType, Polygon};

    fn p(x: f64, y: f64) -> Point {
        Point::float(x, y, 1.0)
    }

    #[test]
    fn detects_zero_area_polygon() {
        let mut cell = Cell::new("top");
        cell.add(Polygon::new(
            [p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        let mut library = Library::new("test");
        library.add_cell(cell);

        let report = validate_library(&library, &DesignRuleOptions::basic());

        assert_eq!(report.violations.len(), 1);
        assert_eq!(
            report.violations[0].kind,
            DesignRuleViolationKind::ZeroAreaPolygon
        );
    }

    #[test]
    fn detects_self_intersecting_polygon() {
        let mut cell = Cell::new("top");
        cell.add(Polygon::new(
            [p(0.0, 0.0), p(2.0, 2.0), p(0.0, 2.0), p(2.0, 0.0)],
            Layer::new(1),
            DataType::new(0),
        ));
        let mut library = Library::new("test");
        library.add_cell(cell);

        let report = validate_library(&library, &DesignRuleOptions::basic());

        assert!(report
            .violations
            .iter()
            .any(|violation| violation.kind == DesignRuleViolationKind::SelfIntersectingPolygon));
    }

    #[test]
    fn detects_zero_length_path() {
        let mut cell = Cell::new("top");
        cell.add(Path::new(
            [p(1.0, 1.0), p(1.0, 1.0)],
            Layer::new(1),
            DataType::new(0),
            Some(PathType::Square),
            Some(Unit::float(1.0, 1.0)),
            None,
            None,
        ));
        let mut library = Library::new("test");
        library.add_cell(cell);

        let report = validate_library(&library, &DesignRuleOptions::basic());

        assert_eq!(
            report.violations[0].kind,
            DesignRuleViolationKind::ZeroLengthPath
        );
    }

    #[test]
    fn detects_path_width_below_minimum() {
        let mut cell = Cell::new("top");
        cell.add(Path::new(
            [p(0.0, 0.0), p(10.0, 0.0)],
            Layer::new(1),
            DataType::new(0),
            Some(PathType::Square),
            Some(Unit::float(1.0, 1.0)),
            None,
            None,
        ));
        let mut library = Library::new("test");
        library.add_cell(cell);
        let options = DesignRuleOptions {
            min_path_width: Some(Unit::float(2.0, 1.0)),
            ..DesignRuleOptions::basic()
        };

        let report = validate_library(&library, &options);

        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.kind == DesignRuleViolationKind::PathWidthBelowMinimum)
        );
    }

    #[test]
    fn detects_same_layer_spacing_below_minimum() {
        let mut cell = Cell::new("top");
        cell.add(Polygon::rectangle(
            p(0.0, 0.0),
            p(1.0, 1.0),
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Polygon::rectangle(
            p(1.5, 0.0),
            p(2.5, 1.0),
            Layer::new(1),
            DataType::new(2),
        ));
        let mut library = Library::new("test");
        library.add_cell(cell);
        let options = DesignRuleOptions {
            min_spacing: Some(Unit::float(1.0, 1.0)),
            ..DesignRuleOptions::basic()
        };

        let report = validate_library(&library, &options);

        assert!(report.violations.iter().any(|violation| {
            violation.kind == DesignRuleViolationKind::SameLayerSpacingBelowMinimum
                && violation.other_element_index == Some(1)
        }));
    }
}
