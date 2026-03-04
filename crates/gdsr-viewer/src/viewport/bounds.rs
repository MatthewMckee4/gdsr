use gdsr::Element;

/// Computes the axis-aligned bounding box of a slice of points as `[min_x, min_y, max_x, max_y]`.
/// Returns `None` if the slice is empty.
pub(crate) fn points_bbox(points: &[gdsr::Point]) -> Option<[f64; 4]> {
    let first = points.first()?;
    let mut min_x = first.x().absolute_value();
    let mut min_y = first.y().absolute_value();
    let mut max_x = min_x;
    let mut max_y = min_y;
    for p in &points[1..] {
        let x = p.x().absolute_value();
        let y = p.y().absolute_value();
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    Some([min_x, min_y, max_x, max_y])
}

/// Returns `true` if two axis-aligned bounding boxes overlap.
pub(crate) fn bbox_overlaps(bbox: &[f64; 4], visible: &[f64; 4]) -> bool {
    bbox[2] >= visible[0] && bbox[0] <= visible[2] && bbox[3] >= visible[1] && bbox[1] <= visible[3]
}

/// Returns the world-space bounding box of a single element as `[min_x, min_y, max_x, max_y]`.
/// Returns `None` for references and elements with no points.
pub fn element_bbox(element: &Element) -> Option<[f64; 4]> {
    match element {
        Element::Polygon(p) => points_bbox(p.points()),
        Element::Path(p) => points_bbox(p.points()),
        Element::Text(t) => {
            let x = t.origin().x().absolute_value();
            let y = t.origin().y().absolute_value();
            Some([x, y, x, y])
        }
        Element::Reference(_) => None,
    }
}

/// Computes the bounding box of the given elements in world coordinates.
/// Returns `None` if there are no geometric elements.
pub fn compute_bounds(elements: &[Element]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut found = false;

    for element in elements {
        if let Some(bbox) = element_bbox(element) {
            min_x = min_x.min(bbox[0]);
            min_y = min_y.min(bbox[1]);
            max_x = max_x.max(bbox[2]);
            max_y = max_y.max(bbox[3]);
            found = true;
        }
    }

    if found {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::helpers::*;
    use gdsr::Point;

    const EPSILON: f64 = 1e-6;

    #[test]
    fn compute_bounds_empty_returns_none() {
        assert!(compute_bounds(&[]).is_none());
    }

    #[test]
    fn compute_bounds_ignores_references() {
        assert!(compute_bounds(&[reference()]).is_none());
    }

    #[test]
    fn compute_bounds_polygon() {
        let elem = polygon(vec![(0, 0), (1000, 0), (1000, 2000)], 1, 0);
        let (min_x, min_y, max_x, max_y) = compute_bounds(&[elem]).expect("should have bounds");
        assert!((min_x - 0.0).abs() < EPSILON);
        assert!((min_y - 0.0).abs() < EPSILON);
        assert!((max_x - 1000.0 * 1e-9).abs() < EPSILON);
        assert!((max_y - 2000.0 * 1e-9).abs() < EPSILON);
    }

    #[test]
    fn compute_bounds_path() {
        let elem = path(vec![(100, 200), (300, 400)], 1, 0, Some(10));
        let (min_x, min_y, max_x, max_y) = compute_bounds(&[elem]).expect("should have bounds");
        assert!((min_x - 100.0 * 1e-9).abs() < EPSILON);
        assert!((min_y - 200.0 * 1e-9).abs() < EPSILON);
        assert!((max_x - 300.0 * 1e-9).abs() < EPSILON);
        assert!((max_y - 400.0 * 1e-9).abs() < EPSILON);
    }

    #[test]
    fn compute_bounds_text() {
        let elem = text("hello", 500, 600, 1);
        let (min_x, min_y, max_x, max_y) = compute_bounds(&[elem]).expect("should have bounds");
        assert!((min_x - 500.0 * 1e-9).abs() < EPSILON);
        assert!((min_y - 600.0 * 1e-9).abs() < EPSILON);
        assert_eq!(min_x, max_x);
        assert_eq!(min_y, max_y);
    }

    #[test]
    fn compute_bounds_mixed_elements() {
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let txt = text("far", 500, 500, 2);
        let (min_x, min_y, _, _) = compute_bounds(&[poly, txt]).expect("should have bounds");
        assert!((min_x - 0.0).abs() < EPSILON);
        assert!((min_y - 0.0).abs() < EPSILON);
    }

    #[test]
    fn points_bbox_empty() {
        assert!(points_bbox(&[]).is_none());
    }

    #[test]
    fn points_bbox_single_point() {
        let points = vec![Point::default_integer(100, 200)];
        let bbox = points_bbox(&points).expect("should have bbox");
        let scale = 1e-9;
        assert!((bbox[0] - 100.0 * scale).abs() < 1e-15);
        assert!((bbox[1] - 200.0 * scale).abs() < 1e-15);
        assert_eq!(bbox[0], bbox[2]);
        assert_eq!(bbox[1], bbox[3]);
    }

    #[test]
    fn bbox_overlaps_fully_contained() {
        assert!(bbox_overlaps(&[1.0, 1.0, 2.0, 2.0], &[0.0, 0.0, 3.0, 3.0]));
    }

    #[test]
    fn bbox_overlaps_disjoint() {
        assert!(!bbox_overlaps(&[0.0, 0.0, 1.0, 1.0], &[2.0, 2.0, 3.0, 3.0]));
    }

    #[test]
    fn bbox_overlaps_touching_edge() {
        assert!(bbox_overlaps(&[0.0, 0.0, 1.0, 1.0], &[1.0, 0.0, 2.0, 1.0]));
    }

    #[test]
    fn bbox_overlaps_is_symmetric() {
        let a = [0.0, 0.0, 2.0, 2.0];
        let b = [1.0, 1.0, 3.0, 3.0];
        assert_eq!(bbox_overlaps(&a, &b), bbox_overlaps(&b, &a));
    }
}
