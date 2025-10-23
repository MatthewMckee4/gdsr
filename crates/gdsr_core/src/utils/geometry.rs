use geo::{
    Area, BoundingRect, Contains, Coord, EuclideanLength, Line, LineString, Point as GeoPoint,
    Polygon,
};

use crate::{Point, Unit};

fn to_geo_float_coords(points: impl IntoIterator<Item = impl Into<Point>>) -> Vec<Coord<f64>> {
    points.into_iter().map(to_geo_float_coord).collect()
}

fn to_geo_float_coord(point: impl Into<Point>) -> Coord<f64> {
    let point: Point = point.into();
    Coord {
        x: point.x().as_float(),
        y: point.y().as_float(),
    }
}

fn to_geo_float_points(points: impl IntoIterator<Item = impl Into<Point>>) -> Vec<GeoPoint<f64>> {
    points.into_iter().map(to_geo_float_point).collect()
}

fn to_geo_float_point(point: impl Into<Point>) -> GeoPoint<f64> {
    to_geo_float_coord(point.into()).into()
}

/// Calculate the bounding box of a collection of points
/// Returns (`min_point`, `max_point`) representing the bottom-left and top-right corners
pub fn bounding_box(points: impl IntoIterator<Item = impl Into<Point>>) -> (Point, Point) {
    let points = points
        .into_iter()
        .map(std::convert::Into::into)
        .collect::<Vec<Point>>();

    if points.is_empty() {
        return (Point::default(), Point::default());
    }

    // Use geo's BoundingRect trait for robust calculation
    let multipoint = geo::MultiPoint::new(to_geo_float_points(&points));
    multipoint.bounding_rect().map_or_else(
        || {
            let first = points[0];
            (first, first)
        },
        |rect| {
            let first_point = points[0];

            let (x_units, y_units) = first_point.units();
            let min_point = Point::new(
                Unit::float(rect.min().x, x_units),
                Unit::float(rect.min().y, y_units),
            );
            let max_point = Point::new(
                Unit::float(rect.max().x, x_units),
                Unit::float(rect.max().y, y_units),
            );
            (min_point, max_point)
        },
    )
}

/// Calculate the area of a polygon defined by points using the shoelace formula
/// Points should be in order (clockwise or counter-clockwise)
pub fn area(points: impl IntoIterator<Item = impl Into<Point>>) -> f64 {
    let points = points.into_iter().map(Into::into).collect::<Vec<Point>>();
    if points.len() < 3 {
        return 0.0;
    }

    let coords = to_geo_float_coords(points);

    // Close the polygon by adding the first point at the end if not already closed
    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    polygon.unsigned_area().abs()
}

/// Calculate the perimeter of a polygon defined by points
/// For open polygons, calculates the total length of all segments
/// For closed polygons, includes the segment from last to first point
pub fn perimeter(points: impl IntoIterator<Item = impl Into<Point>>) -> f64 {
    let points = points
        .into_iter()
        .map(std::convert::Into::into)
        .collect::<Vec<Point>>();
    if points.len() < 2 {
        return 0.0;
    }

    let coords = to_geo_float_coords(points);

    let linestring = LineString::new(coords);

    linestring.euclidean_length()
}

/// Check if a point is inside a polygon using the ray casting algorithm
/// The polygon is defined by an ordered list of points
pub fn is_point_inside(point: &Point, points: impl IntoIterator<Item = impl Into<Point>>) -> bool {
    let points = points
        .into_iter()
        .map(std::convert::Into::into)
        .collect::<Vec<Point>>();
    if points.len() < 3 {
        return false;
    }

    let coords = to_geo_float_coords(points);

    // Ensure the polygon is closed
    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    polygon.contains(&to_geo_float_coord(point))
}

/// Check if a point lies on the edge of a polygon
pub fn is_point_on_edge(point: &Point, points: impl IntoIterator<Item = impl Into<Point>>) -> bool {
    let points = points
        .into_iter()
        .map(std::convert::Into::into)
        .collect::<Vec<Point>>();
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
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn test_bounding_box() {
        let points = vec![
            Point::integer(1, 2, 1e-9),
            Point::integer(4, 6, 1e-9),
            Point::integer(-1, 3, 1e-9),
            Point::integer(2, -1, 1e-9),
        ];

        let (min_point, max_point) = bounding_box(points);
        assert_eq!(min_point, Point::integer(-1, -1, 1e-9));
        assert_eq!(max_point, Point::integer(4, 6, 1e-9));
    }

    #[test]
    fn test_area() {
        // Square with side length 2
        let square = vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)];

        let area_result = area(square);
        assert_relative_eq!(area_result, 4.0, epsilon = 1e-10);
    }

    #[test]
    fn test_point_inside() {
        let square = vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)];

        assert!(is_point_inside(&Point::integer(1, 1, 1e-9), square.clone()));
        assert!(!is_point_inside(&Point::integer(3, 3, 1e-9), square));
    }

    #[test]
    fn test_point_on_edge() {
        let triangle = vec![(0.0, 0.0), (2.0, 0.0), (1.0, 2.0)];

        assert!(is_point_on_edge(
            &Point::integer(1, 0, 1e-9),
            triangle.clone()
        ));
        assert!(!is_point_on_edge(&Point::integer(1, 1, 1e-9), triangle));
    }
}
