use geo::{Area, BoundingRect, Contains, Coord, EuclideanLength, Line, LineString, Polygon};

use crate::{CoordinateUnit, Point};

fn to_geo_coords<T: CoordinateUnit>(points: &[Point<T>]) -> Vec<Coord<f64>> {
    points
        .iter()
        .map(|p| Coord {
            x: p.x().to_float_value(),
            y: p.y().to_float_value(),
        })
        .collect()
}

fn to_geo_points<T: CoordinateUnit>(points: &[Point<T>]) -> Vec<geo::Point<f64>> {
    points
        .iter()
        .map(|p| {
            Coord {
                x: p.x().to_float_value(),
                y: p.y().to_float_value(),
            }
            .into()
        })
        .collect()
}

/// Calculate the bounding box of a collection of points
/// Returns (`min_point`, `max_point`) representing the bottom-left and top-right corners
pub fn bounding_box<T: CoordinateUnit>(points: &[Point<T>]) -> (Point<T>, Point<T>) {
    if points.is_empty() {
        return (
            Point::new(T::zero(), T::zero()),
            Point::new(T::zero(), T::zero()),
        );
    }

    // Use geo's BoundingRect trait for robust calculation
    let multipoint = geo::MultiPoint::new(to_geo_points(points));
    multipoint.bounding_rect().map_or_else(
        || {
            let first = points[0];
            (first, first)
        },
        |rect| {
            let min_point = geo::Point::new(rect.min().x, rect.min().y);
            let max_point = geo::Point::new(rect.max().x, rect.max().y);
            (min_point.into(), max_point.into())
        },
    )
}

/// Calculate the area of a polygon defined by points using the shoelace formula
/// Points should be in order (clockwise or counter-clockwise)
pub fn area<T: CoordinateUnit>(points: &[Point<T>]) -> T {
    if points.len() < 3 {
        return T::zero();
    }

    let coords = to_geo_coords(points);

    // Close the polygon by adding the first point at the end if not already closed
    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    T::from_float_value(polygon.unsigned_area().abs())
}

/// Calculate the perimeter of a polygon defined by points
/// For open polygons, calculates the total length of all segments
/// For closed polygons, includes the segment from last to first point
pub fn perimeter<T: CoordinateUnit>(points: &[Point<T>]) -> T {
    if points.len() < 2 {
        return T::zero();
    }

    let coords = to_geo_coords(points);

    let linestring = LineString::new(coords);

    T::from_float_value(linestring.euclidean_length())
}

/// Check if a point is inside a polygon using the ray casting algorithm
/// The polygon is defined by an ordered list of points
pub fn is_point_inside<T: CoordinateUnit>(point: &Point<T>, polygon_points: &[Point<T>]) -> bool {
    if polygon_points.len() < 3 {
        return false;
    }

    let coords = to_geo_coords(polygon_points);

    // Ensure the polygon is closed
    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    let geo_coord: geo::Coord<f64> = point.into();

    polygon.contains(&geo_coord)
}

/// Check if a point lies on the edge of a polygon
pub fn is_point_on_edge<T: CoordinateUnit>(point: &Point<T>, polygon_points: &[Point<T>]) -> bool {
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
pub fn is_point_on_line_segment<T: CoordinateUnit>(
    point: &Point<T>,
    a: &Point<T>,
    b: &Point<T>,
) -> bool {
    let line_segment = Line::new(
        Coord {
            x: a.x().to_float_value(),
            y: a.y().to_float_value(),
        },
        Coord {
            x: b.x().to_float_value(),
            y: b.y().to_float_value(),
        },
    );

    let geo_coord: geo::Coord<f64> = point.into();

    line_segment.contains(&geo_coord)
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
    use crate::DatabaseFloatUnit;

    #[test]
    fn test_bounding_box() {
        let points = vec![
            Point::new(DatabaseFloatUnit(1.0), DatabaseFloatUnit(2.0)),
            Point::new(DatabaseFloatUnit(4.0), DatabaseFloatUnit(6.0)),
            Point::new(DatabaseFloatUnit(-1.0), DatabaseFloatUnit(3.0)),
            Point::new(DatabaseFloatUnit(2.0), DatabaseFloatUnit(-1.0)),
        ];

        let (min_point, max_point) = bounding_box(&points);
        assert_eq!(
            min_point,
            Point::new(DatabaseFloatUnit(-1.0), DatabaseFloatUnit(-1.0))
        );
        assert_eq!(
            max_point,
            Point::new(DatabaseFloatUnit(4.0), DatabaseFloatUnit(6.0))
        );
    }

    #[test]
    fn test_area() {
        // Square with side length 2
        let square = vec![
            Point::new(DatabaseFloatUnit(0.0), DatabaseFloatUnit(0.0)),
            Point::new(DatabaseFloatUnit(2.0), DatabaseFloatUnit(0.0)),
            Point::new(DatabaseFloatUnit(2.0), DatabaseFloatUnit(2.0)),
            Point::new(DatabaseFloatUnit(0.0), DatabaseFloatUnit(2.0)),
        ];

        let area_result = area(&square);
        assert_relative_eq!(area_result.to_float_value(), 4.0, epsilon = 1e-10);
    }

    #[test]
    fn test_point_inside() {
        let square = vec![
            Point::new(DatabaseFloatUnit(0.0), DatabaseFloatUnit(0.0)),
            Point::new(DatabaseFloatUnit(2.0), DatabaseFloatUnit(0.0)),
            Point::new(DatabaseFloatUnit(2.0), DatabaseFloatUnit(2.0)),
            Point::new(DatabaseFloatUnit(0.0), DatabaseFloatUnit(2.0)),
        ];

        assert!(is_point_inside(
            &Point::new(DatabaseFloatUnit(1.0), DatabaseFloatUnit(1.0)),
            &square
        ));
        assert!(!is_point_inside(
            &Point::new(DatabaseFloatUnit(3.0), DatabaseFloatUnit(3.0)),
            &square
        ));
    }

    #[test]
    fn test_point_on_edge() {
        let triangle = vec![
            Point::new(DatabaseFloatUnit(0.0), DatabaseFloatUnit(0.0)),
            Point::new(DatabaseFloatUnit(2.0), DatabaseFloatUnit(0.0)),
            Point::new(DatabaseFloatUnit(1.0), DatabaseFloatUnit(2.0)),
        ];

        assert!(is_point_on_edge(
            &Point::new(DatabaseFloatUnit(1.0), DatabaseFloatUnit(0.0)),
            &triangle
        ));
        assert!(!is_point_on_edge(
            &Point::new(DatabaseFloatUnit(1.0), DatabaseFloatUnit(1.0)),
            &triangle
        ));
    }
}
