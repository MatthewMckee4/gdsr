use geo::{BooleanOps, OpType};

use crate::clip::{polygon_to_geo, polygons_from_geo};
use crate::{Element, Polygon};

const PATH_ARC_SEGMENTS: usize = 16;

fn apply(left: &Polygon, right: &Polygon, operation: OpType) -> Vec<Polygon> {
    let Some(left_geometry) = polygon_to_geo(left) else {
        return Vec::new();
    };
    let Some(right_geometry) = polygon_to_geo(right) else {
        return Vec::new();
    };

    left_geometry
        .boolean_op(&right_geometry, operation)
        .0
        .iter()
        .flat_map(|polygon| polygons_from_geo(polygon, left))
        .collect()
}

fn element_polygon(element: &Element) -> Option<Polygon> {
    match element {
        Element::Polygon(polygon) => Some(polygon.clone()),
        Element::Box(gds_box) => {
            let mut polygon = Polygon::new(gds_box.points(), gds_box.layer(), gds_box.box_type());
            polygon
                .properties_mut()
                .extend_from_slice(gds_box.properties());
            Some(polygon)
        }
        Element::Path(path) => path.to_polygon_points(PATH_ARC_SEGMENTS).map(|points| {
            let mut polygon = Polygon::new(points, path.layer(), path.data_type());
            polygon
                .properties_mut()
                .extend_from_slice(path.properties());
            polygon
        }),
        Element::Node(_) | Element::Text(_) | Element::Reference(_) => None,
    }
}

fn apply_elements(left: &Element, right: &Element, operation: OpType) -> Option<Vec<Polygon>> {
    let left = element_polygon(left)?;
    let right = element_polygon(right)?;
    Some(apply(&left, &right, operation))
}

impl Polygon {
    /// Returns the filled union of this polygon and `other`.
    ///
    /// Results preserve this polygon's layer, data type, coordinate units, and properties.
    #[must_use]
    pub fn union(&self, other: &Self) -> Vec<Self> {
        apply(self, other, OpType::Union)
    }

    /// Returns the regions shared by this polygon and `other`.
    ///
    /// Results preserve this polygon's layer, data type, coordinate units, and properties.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Vec<Self> {
        apply(self, other, OpType::Intersection)
    }

    /// Returns the regions in this polygon that are not in `other`.
    ///
    /// Results preserve this polygon's layer, data type, coordinate units, and properties.
    #[must_use]
    pub fn difference(&self, other: &Self) -> Vec<Self> {
        apply(self, other, OpType::Difference)
    }

    /// Returns the regions in either polygon, but not both.
    ///
    /// Results preserve this polygon's layer, data type, coordinate units, and properties.
    #[must_use]
    pub fn xor(&self, other: &Self) -> Vec<Self> {
        apply(self, other, OpType::Xor)
    }
}

impl Element {
    /// Returns the filled union of this element and `other`.
    ///
    /// Polygons, boxes, and paths with positive width are supported. Returns `None` when either
    /// element has no filled-area representation. Results preserve this element's layer, data
    /// type, coordinate units, and properties.
    #[must_use]
    pub fn union(&self, other: &Self) -> Option<Vec<Polygon>> {
        apply_elements(self, other, OpType::Union)
    }

    /// Returns the regions shared by this element and `other`.
    ///
    /// Polygons, boxes, and paths with positive width are supported. Returns `None` when either
    /// element has no filled-area representation. Results preserve this element's layer, data
    /// type, coordinate units, and properties.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Vec<Polygon>> {
        apply_elements(self, other, OpType::Intersection)
    }

    /// Returns the regions in this element that are not in `other`.
    ///
    /// Polygons, boxes, and paths with positive width are supported. Returns `None` when either
    /// element has no filled-area representation. Results preserve this element's layer, data
    /// type, coordinate units, and properties.
    #[must_use]
    pub fn difference(&self, other: &Self) -> Option<Vec<Polygon>> {
        apply_elements(self, other, OpType::Difference)
    }

    /// Returns the regions in either element, but not both.
    ///
    /// Polygons, boxes, and paths with positive width are supported. Returns `None` when either
    /// element has no filled-area representation. Results preserve this element's layer, data
    /// type, coordinate units, and properties.
    #[must_use]
    pub fn xor(&self, other: &Self) -> Option<Vec<Polygon>> {
        apply_elements(self, other, OpType::Xor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataType, GdsBox, Layer, Path, Point, Property, Text, Unit};

    const NANOMETERS: f64 = 1e-9;

    fn point(x: f64, y: f64) -> Point {
        Point::float(x, y, NANOMETERS)
    }

    fn rectangle(
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        layer: Layer,
        data_type: DataType,
    ) -> Polygon {
        Polygon::rectangle(point(min_x, min_y), point(max_x, max_y), layer, data_type)
    }

    fn total_area(polygons: &[Polygon]) -> f64 {
        polygons
            .iter()
            .map(|polygon| polygon.area().float_value().abs())
            .sum()
    }

    fn assert_area(polygons: &[Polygon], expected: f64) {
        let error = (total_area(polygons) - expected).abs();
        assert!(
            error < expected.abs().max(1.0) * 1e-8,
            "expected area {expected}, got {}",
            total_area(polygons)
        );
    }

    #[test]
    fn polygon_boolean_operations_return_all_regions() {
        let layer = Layer::new(7);
        let data_type = DataType::new(11);
        let mut left = rectangle(0.0, 0.0, 10.0, 10.0, layer, data_type);
        left.properties_mut().push(Property::new(3, "left"));
        let right = rectangle(5.0, 0.0, 15.0, 10.0, Layer::new(2), DataType::new(3));

        assert_area(&left.union(&right), 150.0);
        assert_area(&left.intersection(&right), 50.0);
        assert_area(&left.difference(&right), 50.0);
        let xor = left.xor(&right);
        assert_eq!(xor.len(), 2);
        assert_area(&xor, 100.0);
        assert!(xor.iter().all(|polygon| polygon.layer() == layer
            && polygon.data_type() == data_type
            && polygon.properties() == left.properties()));
    }

    #[test]
    fn difference_can_return_disjoint_regions() {
        let left = rectangle(0.0, 0.0, 10.0, 10.0, Layer::default(), DataType::default());
        let stripe = rectangle(4.0, -1.0, 6.0, 11.0, Layer::default(), DataType::default());

        let difference = left.difference(&stripe);

        assert_eq!(difference.len(), 2);
        assert_area(&difference, 80.0);
    }

    #[test]
    fn difference_preserves_holes_as_filled_polygons() {
        let left = rectangle(0.0, 0.0, 10.0, 10.0, Layer::default(), DataType::default());
        let center = rectangle(3.0, 3.0, 7.0, 7.0, Layer::default(), DataType::default());

        let difference = left.difference(&center);

        assert!(difference.len() > 1);
        assert_area(&difference, 84.0);
    }

    #[test]
    fn operations_convert_other_coordinate_units() {
        let layer = Layer::new(4);
        let data_type = DataType::new(5);
        let left = rectangle(0.0, 0.0, 1_000.0, 1_000.0, layer, data_type);
        let right = Polygon::rectangle(
            Point::float(0.5, 0.0, 1e-6),
            Point::float(1.5, 1.0, 1e-6),
            Layer::new(9),
            DataType::new(9),
        );

        let intersection = left.intersection(&right);

        assert_area(&intersection, 500_000.0);
        assert!(
            intersection.iter().flat_map(Polygon::points).all(|point| {
                point.x().units() == NANOMETERS && point.y().units() == NANOMETERS
            })
        );
    }

    #[test]
    fn element_operations_support_boxes_and_filled_paths() {
        let layer = Layer::new(6);
        let data_type = DataType::new(8);
        let gds_box = Element::Box(GdsBox::new(
            point(0.0, 0.0),
            point(10.0, 10.0),
            layer,
            data_type,
        ));
        let path = Element::Path(Path::new(
            [point(5.0, -5.0), point(5.0, 15.0)],
            Layer::new(1),
            DataType::new(2),
            None,
            Some(Unit::float(2.0, NANOMETERS)),
            None,
            None,
        ));

        let intersection = gds_box
            .intersection(&path)
            .expect("box and filled path should have area representations");

        assert_area(&intersection, 20.0);
        assert!(
            intersection
                .iter()
                .all(|polygon| polygon.layer() == layer && polygon.data_type() == data_type)
        );
    }

    #[test]
    fn element_operations_reject_elements_without_filled_area() {
        let polygon = Element::Polygon(rectangle(
            0.0,
            0.0,
            10.0,
            10.0,
            Layer::default(),
            DataType::default(),
        ));
        let path_without_width = Element::Path(Path::new(
            [point(0.0, 0.0), point(10.0, 10.0)],
            Layer::default(),
            DataType::default(),
            None,
            None,
            None,
            None,
        ));

        assert_eq!(polygon.union(&path_without_width), None);
        assert_eq!(polygon.intersection(&Element::Text(Text::default())), None);
    }
}
