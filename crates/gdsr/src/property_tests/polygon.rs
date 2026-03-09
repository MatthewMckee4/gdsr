use quickcheck_macros::quickcheck;

use crate::elements::polygon::close_points;
use crate::*;

/// All vertices satisfy the ellipse equation (x-cx)^2/rx^2 + (y-cy)^2/ry^2 ~= 1.
#[quickcheck]
fn ellipse_points_satisfy_ellipse_equation(num_points: usize) -> bool {
    let num_points = (num_points % 100) + 3;
    let center = Point::float(3.0, -2.0, 1e-6);
    let rx = 7.0;
    let ry = 4.0;
    let polygon = Polygon::ellipse(center, rx, ry, num_points, Layer::new(0), DataType::new(0));

    polygon.points().iter().take(num_points).all(|p| {
        let dx = p.x().float_value() - 3.0;
        let dy = p.y().float_value() - (-2.0);
        let val = (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry);
        (val - 1.0).abs() < 1e-9
    })
}

#[quickcheck]
fn area_is_non_negative(polygon: Polygon) -> bool {
    polygon.area().float_value() >= 0.0
}

#[quickcheck]
fn perimeter_is_non_negative(polygon: Polygon) -> bool {
    polygon.perimeter().float_value() >= 0.0
}

#[quickcheck]
fn translation_preserves_area(polygon: Polygon, dx: i32, dy: i32) -> bool {
    let units = polygon.points()[0].units().0;
    let dx = (dx % 10_000).clamp(-10_000, 10_000);
    let dy = (dy % 10_000).clamp(-10_000, 10_000);
    let delta = Point::integer(dx, dy, units);
    let translated = polygon.clone().translate(delta);
    polygon.area() == translated.area()
}

#[quickcheck]
fn translation_preserves_perimeter(polygon: Polygon, dx: i32, dy: i32) -> bool {
    let units = polygon.points()[0].units().0;
    let dx = (dx % 10_000).clamp(-10_000, 10_000);
    let dy = (dy % 10_000).clamp(-10_000, 10_000);
    let delta = Point::integer(dx, dy, units);
    let translated = polygon.clone().translate(delta);
    polygon.perimeter() == translated.perimeter()
}

/// Rotation should preserve area (within floating point tolerance).
#[quickcheck]
fn rotation_preserves_area(polygon: Polygon) -> bool {
    let units = polygon.points()[0].units().0;
    let centre = Point::integer(0, 0, units);
    let rotated = polygon.clone().rotate(Radians::FRAC_PI_2, centre);
    let original_area = polygon.area().float_value();
    let rotated_area = rotated.area().float_value();
    if original_area == 0.0 {
        return rotated_area == 0.0;
    }
    ((original_area - rotated_area) / original_area).abs() < 1e-6
}

#[quickcheck]
fn bounding_box_contains_all_points(polygon: Polygon) -> bool {
    let (min, max) = polygon.bounding_box();
    polygon.points().iter().all(|p| {
        p.x().float_value() >= min.x().float_value()
            && p.x().float_value() <= max.x().float_value()
            && p.y().float_value() >= min.y().float_value()
            && p.y().float_value() <= max.y().float_value()
    })
}

#[quickcheck]
fn close_points_is_idempotent(polygon: Polygon) -> bool {
    let once = close_points(polygon.points().to_vec());
    let twice = close_points(once.clone());
    once == twice
}

#[quickcheck]
fn rectangle_area_equals_width_times_height(x1: i16, y1: i16, x2: i16, y2: i16) -> bool {
    let corner1 = Point::integer(i32::from(x1), i32::from(y1), 1e-9);
    let corner2 = Point::integer(i32::from(x2), i32::from(y2), 1e-9);
    let rect = Polygon::rectangle(corner1, corner2, Layer::new(0), DataType::new(0));

    let width = (f64::from(x2) - f64::from(x1)).abs();
    let height = (f64::from(y2) - f64::from(y1)).abs();
    let expected = width * height;

    (rect.area().float_value() - expected).abs() < 1e-6
}

#[quickcheck]
fn regular_polygon_vertices_equidistant_from_center(num_sides: usize) -> bool {
    let num_sides = (num_sides % 100) + 3;
    let center = Point::float(0.0, 0.0, 1e-6);
    let radius = 10.0;
    let polygon = Polygon::regular_polygon(
        center,
        radius,
        num_sides,
        Radians::new(0.0),
        Layer::new(0),
        DataType::new(0),
    );

    polygon.points().iter().take(num_sides).all(|p| {
        let dx = p.x().float_value();
        let dy = p.y().float_value();
        let dist = (dx * dx + dy * dy).sqrt();
        (dist - radius).abs() < 1e-9
    })
}
