use geo::{
    Area, BoundingRect, Centroid, Contains, Coord, EuclideanDistance, EuclideanLength, GeoFloat,
    Line, LineString, Point, Polygon,
};
use std::iter::Sum;

/// Calculate the bounding box of a collection of points
/// Returns (min_point, max_point) representing the bottom-left and top-right corners
pub fn bounding_box<T: GeoFloat>(points: &Vec<Point<T>>) -> (Point<T>, Point<T>) {
    if points.is_empty() {
        return (
            Point::new(T::zero(), T::zero()),
            Point::new(T::zero(), T::zero()),
        );
    }

    // Use geo's BoundingRect trait for robust calculation
    let multipoint = geo::MultiPoint::new(points.clone());
    if let Some(rect) = multipoint.bounding_rect() {
        let min_point = Point::new(rect.min().x, rect.min().y);
        let max_point = Point::new(rect.max().x, rect.max().y);
        (min_point, max_point)
    } else {
        // Fallback for edge cases
        let first = points[0];
        (first, first)
    }
}

/// Calculate the area of a polygon defined by points using the shoelace formula
/// Points should be in order (clockwise or counter-clockwise)
pub fn area<T: GeoFloat>(points: &[Point<T>]) -> T {
    if points.len() < 3 {
        return T::zero();
    }

    // Convert points to LineString and then to Polygon
    let coords: Vec<Coord<T>> = points
        .iter()
        .map(|p| Coord { x: p.x(), y: p.y() })
        .collect();

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
pub fn perimeter<T: GeoFloat + Sum>(points: &[Point<T>], closed: bool) -> T {
    if points.len() < 2 {
        return T::zero();
    }

    let coords: Vec<Coord<T>> = points
        .iter()
        .map(|p| Coord { x: p.x(), y: p.y() })
        .collect();

    let mut linestring_coords = coords;

    // If it's a closed polygon, ensure it's actually closed
    if closed {
        if let (Some(first), Some(last)) = (linestring_coords.first(), linestring_coords.last()) {
            if first != last {
                linestring_coords.push(*first);
            }
        }
    }

    let linestring = LineString::new(linestring_coords);
    linestring.euclidean_length()
}

/// Calculate the Euclidean distance between two points
pub fn distance_between_points<T: GeoFloat>(point1: &Point<T>, point2: &Point<T>) -> T {
    point1.euclidean_distance(point2)
}

/// Check if a point is inside a polygon using the ray casting algorithm
/// The polygon is defined by an ordered list of points
pub fn is_point_inside<T: GeoFloat>(point: &Point<T>, polygon_points: &[Point<T>]) -> bool {
    if polygon_points.len() < 3 {
        return false;
    }

    // Convert points to a proper Polygon
    let coords: Vec<Coord<T>> = polygon_points
        .iter()
        .map(|p| Coord { x: p.x(), y: p.y() })
        .collect();

    // Ensure the polygon is closed
    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    polygon.contains(point)
}

/// Check if a point lies on the edge of a polygon
pub fn is_point_on_edge<T: GeoFloat>(point: &Point<T>, polygon_points: &[Point<T>]) -> bool {
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
pub fn is_point_on_line_segment<T: GeoFloat>(point: &Point<T>, a: &Point<T>, b: &Point<T>) -> bool {
    let line_segment = Line::new(Coord { x: a.x(), y: a.y() }, Coord { x: b.x(), y: b.y() });
    line_segment.contains(point)
}

/// Round a floating point value to a specified number of decimal places
pub fn round_to_decimals(value: f64, ndigits: u32) -> f64 {
    let factor = 10f64.powi(ndigits as i32);
    (value * factor).round() / factor
}

// Additional utility functions leveraging geo's capabilities

/// Calculate the centroid of a polygon
pub fn centroid<T: GeoFloat>(points: &[Point<T>]) -> Option<Point<T>> {
    if points.len() < 3 {
        return None;
    }

    let coords: Vec<Coord<T>> = points
        .iter()
        .map(|p| Coord { x: p.x(), y: p.y() })
        .collect();

    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    let polygon = Polygon::new(linestring, vec![]);

    polygon.centroid().map(|c| Point::new(c.x(), c.y()))
}

/// Check if two polygons intersect
pub fn polygons_intersect<T: GeoFloat>(points1: &[Point<T>], points2: &[Point<T>]) -> bool {
    use geo::Intersects;

    if points1.len() < 3 || points2.len() < 3 {
        return false;
    }

    let polygon1 = create_polygon_from_points(points1);
    let polygon2 = create_polygon_from_points(points2);

    polygon1.intersects(&polygon2)
}

/// Get the convex hull of a set of points
pub fn convex_hull<T: GeoFloat>(points: &[Point<T>]) -> Vec<Point<T>> {
    use geo::ConvexHull;

    let multipoint = geo::MultiPoint::new(points.to_vec());
    let hull = multipoint.convex_hull();

    hull.exterior()
        .coords()
        .map(|coord| Point::new(coord.x, coord.y))
        .collect()
}

/// Simplify a polygon using the Douglas-Peucker algorithm
pub fn simplify_polygon<T: GeoFloat>(points: &[Point<T>], epsilon: T) -> Vec<Point<T>> {
    use geo::Simplify;

    if points.len() < 3 {
        return points.to_vec();
    }

    let coords: Vec<Coord<T>> = points
        .iter()
        .map(|p| Coord { x: p.x(), y: p.y() })
        .collect();

    let linestring = LineString::new(coords);
    let simplified = linestring.simplify(&epsilon);

    simplified
        .coords()
        .map(|coord| Point::new(coord.x, coord.y))
        .collect()
}

// Helper function to create a polygon from points
fn create_polygon_from_points<T: GeoFloat>(points: &[Point<T>]) -> Polygon<T> {
    let coords: Vec<Coord<T>> = points
        .iter()
        .map(|p| Coord { x: p.x(), y: p.y() })
        .collect();

    let mut closed_coords = coords;
    if let (Some(first), Some(last)) = (closed_coords.first(), closed_coords.last()) {
        if first != last {
            closed_coords.push(*first);
        }
    }

    let linestring = LineString::new(closed_coords);
    Polygon::new(linestring, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

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
    fn test_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);

        let dist = distance_between_points(&p1, &p2);
        assert_relative_eq!(dist, 5.0, epsilon = 1e-10);
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
