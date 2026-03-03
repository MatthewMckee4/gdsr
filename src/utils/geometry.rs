use geo::{
    Area, BoundingRect, Contains, Coord, EuclideanLength, Line, LineString, Point as GeoPoint,
    Polygon,
};

use crate::{Point, Unit, elements::polygon::get_correct_polygon_points_format};

/// Ensure all points have the same units
fn ensure_points_same_units(points: &[Point], new_units: f64) -> Vec<Point> {
    points
        .iter()
        .map(|point| point.scale_units(new_units))
        .collect()
}

fn to_geo_float_coords(points: &[Point]) -> Vec<Coord<f64>> {
    points.iter().map(to_geo_float_coord).collect()
}

fn to_geo_float_coord(point: &Point) -> Coord<f64> {
    Coord {
        x: point.x().float_value(),
        y: point.y().float_value(),
    }
}

fn to_geo_float_points(points: &[Point]) -> Vec<GeoPoint<f64>> {
    points.iter().map(to_geo_float_point).collect()
}

fn to_geo_float_point(point: &Point) -> GeoPoint<f64> {
    to_geo_float_coord(point).into()
}

/// Calculate the bounding box of a collection of points
/// Returns (`min_point`, `max_point`) representing the bottom-left and top-right corners
pub fn bounding_box(points: &[Point]) -> (Point, Point) {
    if points.is_empty() {
        return (Point::default(), Point::default());
    }

    // Use geo's BoundingRect trait for robust calculation
    let multipoint = geo::MultiPoint::new(to_geo_float_points(points));
    let rect = multipoint.bounding_rect().unwrap();
    let first_point = points[0];

    let (x_units, y_units) = first_point.units();
    let min_point = Point::float(rect.min().x, rect.min().y, x_units);
    let max_point = Point::float(rect.max().x, rect.max().y, y_units);
    (min_point, max_point)
}

/// Calculate the area of a polygon defined by points using the shoelace formula
/// Points should be in order (clockwise or counter-clockwise)
pub fn area(points: &[Point]) -> Unit {
    if points.len() < 3 {
        return Unit::default_integer(0);
    }

    let first_point = points[0];
    let units = first_point.units().0;

    let points = ensure_points_same_units(points, units);

    let points_coords = get_correct_polygon_points_format(points.clone());

    let coords = to_geo_float_coords(&points_coords);

    let linestring = LineString::new(coords);
    let polygon = Polygon::new(linestring, vec![]);

    Unit::float(polygon.unsigned_area().abs(), units)
}

/// Calculate the perimeter of a polygon defined by points
/// For open polygons, calculates the total length of all segments
/// For closed polygons, includes the segment from last to first point
pub fn perimeter(points: &[Point]) -> Unit {
    if points.len() < 2 {
        return Unit::default_integer(0);
    }

    let first_point = points[0];
    let units = first_point.units().0;

    let points = ensure_points_same_units(points, units);

    let coords = to_geo_float_coords(&points);

    let linestring = LineString::new(coords);

    let length = linestring.euclidean_length();

    Unit::float(length, units)
}

/// Check if a point is inside a polygon using the ray casting algorithm
/// The polygon is defined by an ordered list of points
pub fn is_point_inside(point: &Point, points: &[Point]) -> bool {
    if points.len() < 3 {
        return false;
    }

    let points = points.to_vec();

    let points_coords = get_correct_polygon_points_format(points.clone());

    let coords = to_geo_float_coords(&points_coords);

    let linestring = LineString::new(coords);
    let polygon = Polygon::new(linestring, vec![]);

    polygon.contains(&to_geo_float_coord(point))
}

/// Check if a point lies on the edge of a polygon
pub fn is_point_on_edge(point: &Point, points: &[Point]) -> bool {
    if points.len() < 2 {
        return false;
    }

    let num_points = points.len();
    for i in 0..num_points {
        let start = &points[i];
        let end = &points[(i + 1) % num_points];

        if is_point_on_line_segment(point, start, end) {
            return true;
        }
    }
    false
}

/// Check if a point lies on a line segment
pub fn is_point_on_line_segment(point: &Point, a: &Point, b: &Point) -> bool {
    let line_segment = Line::new(to_geo_float_coord(a), to_geo_float_coord(b));
    line_segment.contains(&to_geo_float_coord(point))
}

/// Round a floating point value to a specified number of decimal places
pub fn round_to_decimals(value: f64, ndigits: u32) -> f64 {
    let factor = 10f64.powi(ndigits as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bounding_box() {
        let points = vec![
            Point::integer(1, 2, 1e-9),
            Point::integer(4, 6, 1e-9),
            Point::integer(-1, 3, 1e-9),
            Point::integer(2, -1, 1e-9),
        ];

        let (min_point, max_point) = bounding_box(&points);
        assert_eq!(min_point, Point::integer(-1, -1, 1e-9));
        assert_eq!(max_point, Point::integer(4, 6, 1e-9));
    }

    #[test]
    fn test_area() {
        // Square with side length 2
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
        ];

        let area_result = area(&square);
        assert_eq!(area_result, Unit::float(4.0, 1e-6));
    }

    #[test]
    fn test_point_inside() {
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
        ];

        assert!(is_point_inside(&Point::integer(1, 1, 1e-9), &square));
        assert!(!is_point_inside(&Point::integer(3, 3, 1e-9), &square));
    }

    #[test]
    fn test_point_inside_too_few_points() {
        let square = [Point::float(0.0, 0.0, 1e-6), Point::float(2.0, 0.0, 1e-6)];

        assert!(!is_point_inside(&Point::integer(0, 0, 1e-9), &square));
    }

    #[test]
    fn test_point_on_edge() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];

        assert!(is_point_on_edge(&Point::integer(1, 0, 1e-9), &triangle));
        assert!(!is_point_on_edge(&Point::integer(1, 1, 1e-9), &triangle));
    }

    #[test]
    fn test_point_on_edge_too_few_points() {
        let triangle = [Point::float(0.0, 0.0, 1e-6)];

        assert!(!is_point_on_edge(&Point::integer(0, 0, 1e-9), &triangle));
    }

    #[test]
    fn test_perimeter() {
        // Closed square with side length 2 (5 points, first point repeated at end)
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
            Point::float(0.0, 0.0, 1e-6),
        ];

        let perimeter_result = perimeter(&square);
        // Perimeter should be 8.0 (4 sides of length 2)
        assert_eq!(perimeter_result, Unit::float(8.0, 1e-6));
    }

    #[test]
    fn test_area_triangle() {
        // Triangle with base 2 and height 2
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];

        let area_result = area(&triangle);
        // Area should be 2.0 (0.5 * base * height = 0.5 * 2 * 2)
        assert_eq!(area_result, Unit::float(2.0, 1e-6));
    }

    #[test]
    fn test_area_empty_polygon() {
        let empty: [Point; 0] = [];
        assert_eq!(area(&empty), Unit::float(0.0, 1e-6));

        let single_point = [Point::float(1.0, 1.0, 1e-6)];
        assert_eq!(area(&single_point), Unit::float(0.0, 1e-6));

        let two_points = [Point::float(0.0, 0.0, 1e-6), Point::float(1.0, 1.0, 1e-6)];
        assert_eq!(area(&two_points), Unit::float(0.0, 1e-6));
    }

    #[test]
    fn test_perimeter_open_polygon() {
        // Open line with 3 points
        let line = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(3.0, 0.0, 1e-6),
            Point::float(3.0, 4.0, 1e-6),
        ];

        let perimeter_result = perimeter(&line);
        // Should be 3 + 4 = 7 (just the path length, not closed)
        assert_eq!(perimeter_result, Unit::float(7.0, 1e-6));
    }

    #[test]
    fn test_point_inside_edge_cases() {
        let square = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 2.0, 1e-6),
        ];

        // Point on corner (treated as on edge, not inside)
        assert!(!is_point_inside(&Point::float(0.0, 0.0, 1e-6), &square));

        // Point on edge
        assert!(!is_point_inside(&Point::float(1.0, 0.0, 1e-6), &square));

        // Point clearly inside
        assert!(is_point_inside(&Point::float(1.0, 1.0, 1e-6), &square));

        // Point clearly outside
        assert!(!is_point_inside(&Point::float(-1.0, 1.0, 1e-6), &square));
        assert!(!is_point_inside(&Point::float(3.0, 1.0, 1e-6), &square));
    }

    #[test]
    fn test_point_on_edge_midpoint() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];

        // Midpoint of edge should be on edge
        assert!(is_point_on_edge(&Point::float(1.0, 0.0, 1e-6), &triangle));

        // Point clearly not on edge
        assert!(!is_point_on_edge(&Point::float(0.5, 0.5, 1e-6), &triangle));
    }

    #[test]
    fn test_is_point_on_line_segment() {
        let a = Point::float(0.0, 0.0, 1e-6);
        let b = Point::float(2.0, 2.0, 1e-6);

        // Point on the line (midpoint)
        assert!(is_point_on_line_segment(
            &Point::float(1.0, 1.0, 1e-6),
            &a,
            &b
        ));

        // Point not on the line
        assert!(!is_point_on_line_segment(
            &Point::float(1.0, 0.0, 1e-6),
            &a,
            &b
        ));

        // Point on line but outside segment
        assert!(!is_point_on_line_segment(
            &Point::float(3.0, 3.0, 1e-6),
            &a,
            &b
        ));
    }

    #[test]
    fn test_bounding_box_empty() {
        let empty: [Point; 0] = [];
        let (min, max) = bounding_box(&empty);
        assert_eq!(min, Point::default());
        assert_eq!(max, Point::default());
    }

    #[test]
    fn test_bounding_box_single_point() {
        let points = [Point::integer(5, 3, 1e-9)];
        let (min, max) = bounding_box(&points);
        assert_eq!(min, Point::integer(5, 3, 1e-9));
        assert_eq!(max, Point::integer(5, 3, 1e-9));
    }

    #[test]
    fn test_bounding_box_collinear_points() {
        let points = vec![
            Point::integer(1, 5, 1e-9),
            Point::integer(3, 5, 1e-9),
            Point::integer(7, 5, 1e-9),
        ];
        let (min, max) = bounding_box(&points);
        assert_eq!(min, Point::integer(1, 5, 1e-9));
        assert_eq!(max, Point::integer(7, 5, 1e-9));
    }

    #[test]
    fn test_bounding_box_large_coordinates() {
        let points = vec![
            Point::integer(1_000_000, -2_000_000, 1e-9),
            Point::integer(-3_000_000, 4_000_000, 1e-9),
        ];
        let (min, max) = bounding_box(&points);
        assert_eq!(min, Point::integer(-3_000_000, -2_000_000, 1e-9));
        assert_eq!(max, Point::integer(1_000_000, 4_000_000, 1e-9));
    }

    #[test]
    fn test_area_concave_polygon() {
        let concave = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(4.0, 0.0, 1e-6),
            Point::float(4.0, 4.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 4.0, 1e-6),
        ];
        let area_result = area(&concave);
        assert_eq!(area_result, Unit::float(12.0, 1e-6));
    }

    #[test]
    fn test_perimeter_triangle() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(3.0, 0.0, 1e-6),
            Point::float(0.0, 4.0, 1e-6),
            Point::float(0.0, 0.0, 1e-6),
        ];
        let perimeter_result = perimeter(&triangle);
        assert_eq!(perimeter_result, Unit::float(12.0, 1e-6));
    }

    #[test]
    fn test_perimeter_single_point() {
        let points = [Point::integer(1, 1, 1e-9)];
        assert_eq!(perimeter(&points), Unit::float(0.0, 1e-6));
    }

    #[test]
    fn test_perimeter_two_points() {
        let points = [Point::float(0.0, 0.0, 1e-6), Point::float(3.0, 4.0, 1e-6)];
        let perimeter_result = perimeter(&points);
        assert_eq!(perimeter_result, Unit::float(5.0, 1e-6));
    }

    #[test]
    fn test_is_point_inside_concave_polygon() {
        let concave = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(4.0, 0.0, 1e-6),
            Point::float(4.0, 4.0, 1e-6),
            Point::float(2.0, 2.0, 1e-6),
            Point::float(0.0, 4.0, 1e-6),
        ];
        assert!(is_point_inside(&Point::float(1.0, 1.0, 1e-6), &concave));
        assert!(!is_point_inside(&Point::float(2.0, 3.0, 1e-6), &concave));
    }

    /// Vertices are not considered "on edge" because geo's `Line::contains` excludes endpoints.
    #[test]
    fn test_is_point_on_edge_vertex() {
        let triangle = [
            Point::float(0.0, 0.0, 1e-6),
            Point::float(2.0, 0.0, 1e-6),
            Point::float(1.0, 2.0, 1e-6),
        ];
        assert!(!is_point_on_edge(&Point::float(0.0, 0.0, 1e-6), &triangle));
        assert!(!is_point_on_edge(&Point::float(2.0, 0.0, 1e-6), &triangle));
        assert!(!is_point_on_edge(&Point::float(1.0, 2.0, 1e-6), &triangle));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_round_to_decimals() {
        assert_eq!(round_to_decimals(3.14159, 2), 3.14);
        assert_eq!(round_to_decimals(3.14159, 3), 3.142);
        assert_eq!(round_to_decimals(3.14159, 0), 3.0);
        assert_eq!(round_to_decimals(-2.5678, 2), -2.57);
    }
}
