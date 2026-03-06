use quickcheck_macros::quickcheck;

use crate::config::gds_file_types::GDSRecord;
use crate::geometry;
use crate::traits::ToGds;
use crate::utils::io::RecordReader;
use crate::*;
use std::io::{BufReader, Cursor};

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

/// Verifies that BgnExtn/EndExtn records appear in serialized output iff the path has extensions.
#[quickcheck]
fn serialized_path_contains_extension_records(path: Path) -> bool {
    if path.points().len() < 2 {
        return true;
    }
    let Ok(bytes) = path.to_gds_impl(1e-9) else {
        return true;
    };

    let reader = RecordReader::new(BufReader::new(Cursor::new(&bytes)));
    let mut has_bgn_extn = false;
    let mut has_end_extn = false;
    for record in reader {
        let Ok((rec, _)) = record else { continue };
        match rec {
            GDSRecord::BgnExtn => has_bgn_extn = true,
            GDSRecord::EndExtn => has_end_extn = true,
            _ => {}
        }
    }

    has_bgn_extn == path.begin_extension().is_some()
        && has_end_extn == path.end_extension().is_some()
}

#[quickcheck]
fn to_polygon_returns_none_without_width(path: Path) -> bool {
    let no_width = Path::new(
        path.points().to_vec(),
        path.layer(),
        path.data_type(),
        *path.path_type(),
        None,
        path.begin_extension(),
        path.end_extension(),
    );
    no_width.to_polygon_points(8).is_none()
}

/// For a straight-line path with Square type, expanded polygon area ≈ length × width.
#[quickcheck]
fn straight_path_area_matches_length_times_width(x_end: i16, width: u8) -> bool {
    let x_end = i32::from(x_end).max(1);
    let width = i32::from(width).max(1);
    let units = 1e-6;

    let path = Path::new(
        vec![Point::integer(0, 0, units), Point::integer(x_end, 0, units)],
        Layer::new(0),
        DataType::new(0),
        Some(PathType::Square),
        Some(Unit::integer(width, units)),
        None,
        None,
    );

    let Some(poly_pts) = path.to_polygon_points(8) else {
        return false;
    };

    // area() returns Unit::float(value, units) where value is in scaled coords
    let computed_area = geometry::area(&poly_pts).float_value();
    // In scaled coordinates: length = x_end, width = width
    let expected = f64::from(x_end) * f64::from(width);
    let rel_err = (computed_area - expected).abs() / expected.max(1e-10);
    rel_err < 1e-6
}

/// For a straight Overlap path, area ≈ (length + `begin_ext` + `end_ext`) × width.
#[quickcheck]
fn overlap_extension_area_matches_extended_length_times_width(
    x_end: i16,
    width: u8,
    begin_ext: u8,
    end_ext: u8,
) -> bool {
    let x_end = i32::from(x_end).max(1);
    let width = i32::from(width).max(1);
    let begin_ext = i32::from(begin_ext);
    let end_ext = i32::from(end_ext);
    let units = 1e-6;

    let path = Path::new(
        vec![Point::integer(0, 0, units), Point::integer(x_end, 0, units)],
        Layer::new(0),
        DataType::new(0),
        Some(PathType::Overlap),
        Some(Unit::integer(width, units)),
        Some(Unit::integer(begin_ext, units)),
        Some(Unit::integer(end_ext, units)),
    );

    let Some(poly_pts) = path.to_polygon_points(8) else {
        return false;
    };

    let computed_area = geometry::area(&poly_pts).float_value();
    let expected =
        (f64::from(x_end) + f64::from(begin_ext) + f64::from(end_ext)) * f64::from(width);
    let rel_err = (computed_area - expected).abs() / expected.max(1e-10);
    rel_err < 1e-6
}

/// Overlap with zero extensions produces the same polygon as Square with no extensions.
#[quickcheck]
fn overlap_zero_extension_matches_square(x_end: i16, width: u8) -> bool {
    let x_end = i32::from(x_end).max(1);
    let width = i32::from(width).max(1);
    let units = 1e-6;

    let overlap_path = Path::new(
        vec![Point::integer(0, 0, units), Point::integer(x_end, 0, units)],
        Layer::new(0),
        DataType::new(0),
        Some(PathType::Overlap),
        Some(Unit::integer(width, units)),
        Some(Unit::integer(0, units)),
        Some(Unit::integer(0, units)),
    );

    let square_path = Path::new(
        vec![Point::integer(0, 0, units), Point::integer(x_end, 0, units)],
        Layer::new(0),
        DataType::new(0),
        Some(PathType::Square),
        Some(Unit::integer(width, units)),
        None,
        None,
    );

    let overlap_pts = overlap_path.to_polygon_points(8);
    let square_pts = square_path.to_polygon_points(8);
    overlap_pts == square_pts
}

/// All centerline points of a straight horizontal path should be inside the expanded polygon.
#[quickcheck]
fn centerline_points_inside_expanded_straight_path(x_end: i16, width: u8) -> bool {
    let x_end = i32::from(x_end).max(1);
    let width = i32::from(width).max(1);
    let units = 1e-6;

    let path = Path::new(
        vec![Point::integer(0, 0, units), Point::integer(x_end, 0, units)],
        Layer::new(0),
        DataType::new(0),
        Some(PathType::Square),
        Some(Unit::integer(width, units)),
        None,
        None,
    );

    let Some(poly_pts) = path.to_polygon_points(16) else {
        return false;
    };
    if poly_pts.len() < 3 {
        return true;
    }

    path.points().iter().all(|p| {
        geometry::is_point_inside(p, &poly_pts) || geometry::is_point_on_edge(p, &poly_pts)
    })
}
