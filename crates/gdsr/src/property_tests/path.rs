use quickcheck_macros::quickcheck;

use crate::config::gds_file_types::GDSRecord;
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
