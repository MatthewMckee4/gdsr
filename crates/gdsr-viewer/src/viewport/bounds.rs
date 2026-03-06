use gdsr::Element;

use crate::drawable::{Drawable, WorldBBox};

/// Computes the bounding box of the given elements in world coordinates.
/// Returns `None` if there are no geometric elements.
pub fn compute_bounds(elements: &[Element]) -> Option<WorldBBox> {
    let mut result: Option<WorldBBox> = None;

    for element in elements {
        if let Some(bbox) = element.world_bbox() {
            result = Some(match result {
                Some(acc) => acc.merge(&bbox),
                None => bbox,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::helpers::*;

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
        let bounds = compute_bounds(&[elem]).expect("should have bounds");
        assert!((bounds.min_x - 0.0).abs() < EPSILON);
        assert!((bounds.min_y - 0.0).abs() < EPSILON);
        assert!((bounds.max_x - 1000.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.max_y - 2000.0 * 1e-9).abs() < EPSILON);
    }

    #[test]
    fn compute_bounds_path() {
        let elem = path(vec![(100, 200), (300, 400)], 1, 0, Some(10));
        let bounds = compute_bounds(&[elem]).expect("should have bounds");
        assert!((bounds.min_x - 100.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.min_y - 200.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.max_x - 300.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.max_y - 400.0 * 1e-9).abs() < EPSILON);
    }

    #[test]
    fn compute_bounds_text() {
        let elem = text("hello", 500, 600, 1);
        let bounds = compute_bounds(&[elem]).expect("should have bounds");
        assert!((bounds.min_x - 500.0 * 1e-9).abs() < EPSILON);
        assert!((bounds.min_y - 600.0 * 1e-9).abs() < EPSILON);
        assert_eq!(bounds.min_x, bounds.max_x);
        assert_eq!(bounds.min_y, bounds.max_y);
    }

    #[test]
    fn compute_bounds_mixed_elements() {
        let poly = polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0);
        let txt = text("far", 500, 500, 2);
        let bounds = compute_bounds(&[poly, txt]).expect("should have bounds");
        assert!((bounds.min_x - 0.0).abs() < EPSILON);
        assert!((bounds.min_y - 0.0).abs() < EPSILON);
    }

    #[test]
    fn bbox_overlaps_fully_contained() {
        let a = WorldBBox::new(1.0, 1.0, 2.0, 2.0);
        let b = WorldBBox::new(0.0, 0.0, 3.0, 3.0);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn bbox_overlaps_disjoint() {
        let a = WorldBBox::new(0.0, 0.0, 1.0, 1.0);
        let b = WorldBBox::new(2.0, 2.0, 3.0, 3.0);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn bbox_overlaps_touching_edge() {
        let a = WorldBBox::new(0.0, 0.0, 1.0, 1.0);
        let b = WorldBBox::new(1.0, 0.0, 2.0, 1.0);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn compute_bounds_single_element_with_extreme_coords() {
        let max = i32::MAX / 2;
        let elem = polygon(vec![(0, 0), (max, 0), (max, max)], 1, 0);
        let bounds = compute_bounds(&[elem]).expect("should have bounds");
        assert!(bounds.min_x.is_finite());
        assert!(bounds.max_x.is_finite());
        assert!(bounds.max_x > bounds.min_x);
    }

    #[test]
    fn compute_bounds_many_elements() {
        let elems: Vec<gdsr::Element> = (0..100)
            .map(|i| {
                polygon(
                    vec![
                        (i * 10, i * 10),
                        (i * 10 + 5, i * 10),
                        (i * 10 + 5, i * 10 + 5),
                    ],
                    1,
                    0,
                )
            })
            .collect();
        let bounds = compute_bounds(&elems).expect("should have bounds");
        assert!(bounds.min_x.is_finite());
        assert!(bounds.max_x > bounds.min_x);
        assert!(bounds.max_y > bounds.min_y);
    }

    #[test]
    fn compute_bounds_with_node() {
        let n = gdsr::Node::new(
            vec![
                gdsr::Point::default_integer(100, 200),
                gdsr::Point::default_integer(300, 400),
            ],
            gdsr::Layer::new(1),
            gdsr::DataType::new(0),
        );
        let bounds = compute_bounds(&[gdsr::Element::Node(n)]).expect("should have bounds");
        let scale = 1e-9;
        assert!((bounds.min_x - 100.0 * scale).abs() < EPSILON);
        assert!((bounds.min_y - 200.0 * scale).abs() < EPSILON);
    }

    #[test]
    fn compute_bounds_with_gds_box() {
        let b = gdsr::GdsBox::new(
            gdsr::Point::default_integer(10, 20),
            gdsr::Point::default_integer(30, 40),
            gdsr::Layer::new(1),
            gdsr::DataType::new(0),
        );
        let bounds = compute_bounds(&[gdsr::Element::Box(b)]).expect("should have bounds");
        let scale = 1e-9;
        assert!((bounds.min_x - 10.0 * scale).abs() < EPSILON);
        assert!((bounds.max_x - 30.0 * scale).abs() < EPSILON);
    }

    #[test]
    fn bbox_overlaps_is_symmetric() {
        let a = WorldBBox::new(0.0, 0.0, 2.0, 2.0);
        let b = WorldBBox::new(1.0, 1.0, 3.0, 3.0);
        assert_eq!(a.overlaps(&b), b.overlaps(&a));
    }
}
