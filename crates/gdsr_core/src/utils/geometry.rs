use geo::{Area, BoundingRect, Contains, Coord, EuclideanLength, Line, LineString, Point, Polygon};

use crate::{CoordNum, DatabaseFloatUnit, utils::general::point_to_database_float};

fn to_float_coords<DatabaseUnitT: CoordNum>(
    points: &[Point<DatabaseUnitT>],
) -> Vec<Coord<DatabaseFloatUnit>> {
    points
        .iter()
        .map(|p| Coord {
            x: p.x().to_float(),
            y: p.y().to_float(),
        })
        .collect()
}

/// Calculate the bounding box of a collection of points
/// Returns (`min_point`, `max_point`) representing the bottom-left and top-right corners
pub fn bounding_box<DatabaseUnitT: CoordNum>(
    points: &[Point<DatabaseUnitT>],
) -> (Point<DatabaseUnitT>, Point<DatabaseUnitT>) {
    if points.is_empty() {
        return (
            Point::new(DatabaseUnitT::zero(), DatabaseUnitT::zero()),
            Point::new(DatabaseUnitT::zero(), DatabaseUnitT::zero()),
        );
    }

    // Use geo's BoundingRect trait for robust calculation
    let multipoint = geo::MultiPoint::new(points.to_vec());
    multipoint.bounding_rect().map_or_else(
        || {
            let first = points[0];
            (first, first)
        },
        |rect| {
            let min_point = Point::new(rect.min().x, rect.min().y);
            let max_point = Point::new(rect.max().x, rect.max().y);
            (min_point, max_point)
        },
    )
}

/// Calculate the area of a polygon defined by points using the shoelace formula
/// Points should be in order (clockwise or counter-clockwise)
pub fn area<DatabaseUnitT: CoordNum>(points: &[Point<DatabaseUnitT>]) -> DatabaseUnitT {
    if points.len() < 3 {
        return DatabaseUnitT::zero();
    }

    let coords = to_float_coords(points);

    // Close the polygon by adding the first point at the end if not already closed
    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    DatabaseUnitT::from_float(polygon.unsigned_area().abs())
}

/// Calculate the perimeter of a polygon defined by points
/// For open polygons, calculates the total length of all segments
/// For closed polygons, includes the segment from last to first point
pub fn perimeter<DatabaseUnitT: CoordNum>(points: &[Point<DatabaseUnitT>]) -> DatabaseUnitT {
    if points.len() < 2 {
        return DatabaseUnitT::zero();
    }

    let coords = to_float_coords(points);

    let linestring = LineString::new(coords);

    DatabaseUnitT::from_float(linestring.euclidean_length())
}

/// Check if a point is inside a polygon using the ray casting algorithm
/// The polygon is defined by an ordered list of points
pub fn is_point_inside<DatabaseUnitT: CoordNum>(
    point: &Point<DatabaseUnitT>,
    polygon_points: &[Point<DatabaseUnitT>],
) -> bool {
    if polygon_points.len() < 3 {
        return false;
    }

    let coords = to_float_coords(polygon_points);

    // Ensure the polygon is closed
    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    polygon.contains(&point_to_database_float(*point))
}

/// Check if a point lies on the edge of a polygon
pub fn is_point_on_edge<T: CoordNum>(point: &Point<T>, polygon_points: &[Point<T>]) -> bool {
    if polygon_points.len() < 2 {
        return false;
    }

    let num_points = polygon_points.len();
    for i in 0..num_points {
        let start = &polygon_points[i];
        let end = &polygon_points[(i + 1) % num_points];

        if is_point_on_line_segment(point, start, end) {
            return true;
        }
    }
    false
}

/// Check if a point lies on a line segment
pub fn is_point_on_line_segment<T: CoordNum>(point: &Point<T>, a: &Point<T>, b: &Point<T>) -> bool {
    let line_segment = Line::new(
        Coord {
            x: a.x().to_float(),
            y: a.y().to_float(),
        },
        Coord {
            x: b.x().to_float(),
            y: b.y().to_float(),
        },
    );
    line_segment.contains(&point_to_database_float(*point))
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
            Point::new(1.0, 2.0),
            Point::new(4.0, 6.0),
            Point::new(-1.0, 3.0),
            Point::new(2.0, -1.0),
        ];

        let (min_point, max_point) = bounding_box(&points);
        assert_eq!(min_point, Point::new(-1.0, -1.0));
        assert_eq!(max_point, Point::new(4.0, 6.0));
    }

    #[test]
    fn test_area() {
        // Square with side length 2
        let square = vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(0.0, 2.0),
        ];

        let area_result = area(&square);
        assert_relative_eq!(area_result, 4.0, epsilon = 1e-10);
    }

    #[test]
    fn test_point_inside() {
        let square = vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(2.0, 2.0),
            Point::new(0.0, 2.0),
        ];

        assert!(is_point_inside(&Point::new(1.0, 1.0), &square));
        assert!(!is_point_inside(&Point::new(3.0, 3.0), &square));
    }

    #[test]
    fn test_point_on_edge() {
        let triangle = vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(1.0, 2.0),
        ];

        assert!(is_point_on_edge(&Point::new(1.0, 0.0), &triangle));
        assert!(!is_point_on_edge(&Point::new(1.0, 1.0), &triangle));
    }
}
