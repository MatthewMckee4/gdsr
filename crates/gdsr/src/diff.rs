use crate::Unit;
use crate::{
    Element, GdsBox, Grid, Instance, Library, Node, Path, Point, Polygon, Reference, Text,
};

/// Configuration for comparing two [`Library`] values.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LibraryDiffOptions {
    /// Maximum absolute coordinate difference to ignore, in physical units.
    ///
    /// For example, `1e-9` permits a one-nanometre coordinate difference. Negative
    /// values are treated as zero.
    pub coordinate_tolerance: f64,
}

/// Structured differences between two GDS libraries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibraryDiff {
    /// Cells present only in the newer library.
    pub added_cells: Vec<String>,
    /// Cells present only in the older library.
    pub removed_cells: Vec<String>,
    /// Cells present in both libraries whose ordered elements differ.
    pub modified_cells: Vec<CellDiff>,
}

impl LibraryDiff {
    /// Returns `true` when the libraries have the same cells and elements.
    pub fn is_empty(&self) -> bool {
        self.added_cells.is_empty()
            && self.removed_cells.is_empty()
            && self.modified_cells.is_empty()
    }
}

/// Element-level differences for a cell present in both libraries.
#[derive(Clone, Debug, PartialEq)]
pub struct CellDiff {
    /// Name of the modified cell.
    pub cell_name: String,
    /// Ordered element changes in this cell.
    pub elements: Vec<ElementDiff>,
}

/// A change at an element index in a cell's ordered element sequence.
#[derive(Clone, Debug, PartialEq)]
pub enum ElementDiff {
    /// An element exists only in the newer cell.
    Added { index: usize, element: Element },
    /// An element exists only in the older cell.
    Removed { index: usize, element: Element },
    /// The element at this index changed.
    Modified {
        index: usize,
        before: Box<Element>,
        after: Box<Element>,
    },
}

impl Library {
    /// Compares this library's cells and elements with `other`, treating this value as
    /// the older version. Library names and metadata are not compared.
    pub fn diff(&self, other: &Self, options: LibraryDiffOptions) -> LibraryDiff {
        let coordinate_tolerance = options.coordinate_tolerance.max(0.0);
        let mut added_cells: Vec<String> = other
            .cells()
            .keys()
            .filter(|name| !self.cells().contains_key(name.as_str()))
            .cloned()
            .collect();
        let mut removed_cells: Vec<String> = self
            .cells()
            .keys()
            .filter(|name| !other.cells().contains_key(name.as_str()))
            .cloned()
            .collect();
        added_cells.sort();
        removed_cells.sort();

        let mut common_cells: Vec<_> = self
            .cells()
            .iter()
            .filter_map(|(cell_name, before)| {
                other
                    .get_cell(cell_name)
                    .map(|after| (cell_name, before, after))
            })
            .collect();
        common_cells.sort_by_key(|(cell_name, _, _)| *cell_name);

        let modified_cells = common_cells
            .into_iter()
            .filter_map(|(cell_name, before, after)| {
                let elements =
                    diff_elements(before.elements(), after.elements(), coordinate_tolerance);
                (!elements.is_empty()).then(|| CellDiff {
                    cell_name: cell_name.clone(),
                    elements,
                })
            })
            .collect();

        LibraryDiff {
            added_cells,
            removed_cells,
            modified_cells,
        }
    }
}

fn diff_elements(before: &[Element], after: &[Element], tolerance: f64) -> Vec<ElementDiff> {
    let common_length = before.len().min(after.len());
    let mut differences: Vec<ElementDiff> = before
        .iter()
        .zip(after)
        .enumerate()
        .filter(|(_, (before, after))| !elements_equal(before, after, tolerance))
        .map(|(index, (before, after))| ElementDiff::Modified {
            index,
            before: Box::new(before.clone()),
            after: Box::new(after.clone()),
        })
        .collect();

    differences.extend(
        before
            .iter()
            .enumerate()
            .skip(common_length)
            .map(|(index, element)| ElementDiff::Removed {
                index,
                element: element.clone(),
            }),
    );
    differences.extend(
        after
            .iter()
            .enumerate()
            .skip(common_length)
            .map(|(index, element)| ElementDiff::Added {
                index,
                element: element.clone(),
            }),
    );
    differences
}

fn elements_equal(left: &Element, right: &Element, tolerance: f64) -> bool {
    match (left, right) {
        (Element::Path(left), Element::Path(right)) => paths_equal(left, right, tolerance),
        (Element::Polygon(left), Element::Polygon(right)) => polygons_equal(left, right, tolerance),
        (Element::Box(left), Element::Box(right)) => boxes_equal(left, right, tolerance),
        (Element::Node(left), Element::Node(right)) => nodes_equal(left, right, tolerance),
        (Element::Text(left), Element::Text(right)) => texts_equal(left, right, tolerance),
        (Element::Reference(left), Element::Reference(right)) => {
            references_equal(left, right, tolerance)
        }
        _ => false,
    }
}

fn paths_equal(left: &Path, right: &Path, tolerance: f64) -> bool {
    left.layer() == right.layer()
        && left.data_type() == right.data_type()
        && left.path_type() == right.path_type()
        && points_equal(left.points(), right.points(), tolerance)
        && optional_units_equal(left.width(), right.width(), tolerance)
        && optional_units_equal(left.begin_extension(), right.begin_extension(), tolerance)
        && optional_units_equal(left.end_extension(), right.end_extension(), tolerance)
        && left.properties() == right.properties()
}

fn polygons_equal(left: &Polygon, right: &Polygon, tolerance: f64) -> bool {
    left.layer() == right.layer()
        && left.data_type() == right.data_type()
        && points_equal(left.points(), right.points(), tolerance)
        && left.properties() == right.properties()
}

fn boxes_equal(left: &GdsBox, right: &GdsBox, tolerance: f64) -> bool {
    left.layer() == right.layer()
        && left.box_type() == right.box_type()
        && points_equal(
            &[left.bottom_left(), left.top_right()],
            &[right.bottom_left(), right.top_right()],
            tolerance,
        )
        && left.properties() == right.properties()
}

fn nodes_equal(left: &Node, right: &Node, tolerance: f64) -> bool {
    left.layer() == right.layer()
        && left.node_type() == right.node_type()
        && points_equal(left.points(), right.points(), tolerance)
        && left.properties() == right.properties()
}

fn texts_equal(left: &Text, right: &Text, tolerance: f64) -> bool {
    left.text() == right.text()
        && left.layer() == right.layer()
        && left.data_type() == right.data_type()
        && left.magnification() == right.magnification()
        && left.angle() == right.angle()
        && left.x_reflection() == right.x_reflection()
        && left.vertical_presentation() == right.vertical_presentation()
        && left.horizontal_presentation() == right.horizontal_presentation()
        && points_equal(&[*left.origin()], &[*right.origin()], tolerance)
        && left.properties() == right.properties()
}

fn references_equal(left: &Reference, right: &Reference, tolerance: f64) -> bool {
    instances_equal(left.instance(), right.instance(), tolerance)
        && grids_equal(left.grid(), right.grid(), tolerance)
        && left.properties() == right.properties()
}

fn instances_equal(left: &Instance, right: &Instance, tolerance: f64) -> bool {
    match (left, right) {
        (Instance::Cell(left), Instance::Cell(right)) => left == right,
        (Instance::Element(left), Instance::Element(right)) => {
            elements_equal(left.as_ref().as_ref(), right.as_ref().as_ref(), tolerance)
        }
        _ => false,
    }
}

fn grids_equal(left: &Grid, right: &Grid, tolerance: f64) -> bool {
    left.columns() == right.columns()
        && left.rows() == right.rows()
        && left.magnification() == right.magnification()
        && left.angle() == right.angle()
        && left.x_reflection() == right.x_reflection()
        && points_equal(&[left.origin()], &[right.origin()], tolerance)
        && optional_points_equal(left.spacing_x(), right.spacing_x(), tolerance)
        && optional_points_equal(left.spacing_y(), right.spacing_y(), tolerance)
}

fn points_equal(left: &[Point], right: &[Point], tolerance: f64) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            units_equal(left.x(), right.x(), tolerance)
                && units_equal(left.y(), right.y(), tolerance)
        })
}

fn optional_points_equal(left: Option<Point>, right: Option<Point>, tolerance: f64) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => points_equal(&[left], &[right], tolerance),
        (None, None) => true,
        _ => false,
    }
}

fn optional_units_equal(left: Option<Unit>, right: Option<Unit>, tolerance: f64) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => units_equal(left, right, tolerance),
        (None, None) => true,
        _ => false,
    }
}

fn units_equal(left: Unit, right: Unit, tolerance: f64) -> bool {
    let left = left.absolute_value();
    let right = right.absolute_value();
    left == right || (left - right).abs() <= tolerance
}

#[cfg(test)]
mod tests {
    use crate::{
        Cell, DataType, GdsBox, Grid, HorizontalPresentation, Layer, Node, Path, Point, Polygon,
        Property, Radians, Reference, Text, Unit, VerticalPresentation,
    };

    use super::*;

    fn library_with_cell(cell: Cell) -> Library {
        let mut library = Library::new("library");
        library.add_cell(cell);
        library
    }

    fn polygon(x: f64) -> Polygon {
        Polygon::new(
            [
                Point::float(x, 0.0, 1e-6),
                Point::float(1.0, 0.0, 1e-6),
                Point::float(1.0, 1.0, 1e-6),
            ],
            Layer::new(1),
            DataType::new(0),
        )
    }

    #[test]
    fn identical_libraries_have_an_empty_diff() {
        let library = library_with_cell(Cell::new("top"));

        assert!(
            library
                .diff(&library, LibraryDiffOptions::default())
                .is_empty()
        );
    }

    #[test]
    fn property_changes_are_reported() {
        let mut before_cell = Cell::new("top");
        before_cell.add(polygon(0.0));
        let before = library_with_cell(before_cell);

        let mut changed = polygon(0.0);
        changed.properties_mut().push(Property::new(7, "net-a"));
        let mut after_cell = Cell::new("top");
        after_cell.add(changed);
        let after = library_with_cell(after_cell);

        assert!(matches!(
            before
                .diff(&after, LibraryDiffOptions::default())
                .modified_cells[0]
                .elements
                .as_slice(),
            [ElementDiff::Modified { index: 0, .. }]
        ));
    }

    #[test]
    fn diff_reports_sorted_cell_and_element_changes() {
        let mut before = Library::new("before");
        before.add_cell(Cell::new("removed_b"));
        before.add_cell(Cell::new("removed_a"));
        let mut modified_before = Cell::new("modified");
        modified_before.add(polygon(0.0));
        modified_before.add(polygon(2.0));
        before.add_cell(modified_before);

        let mut after = Library::new("after");
        after.add_cell(Cell::new("added_b"));
        after.add_cell(Cell::new("added_a"));
        let mut modified_after = Cell::new("modified");
        modified_after.add(polygon(0.0));
        modified_after.add(polygon(3.0));
        modified_after.add(polygon(4.0));
        after.add_cell(modified_after);

        let diff = before.diff(&after, LibraryDiffOptions::default());

        assert_eq!(diff.added_cells, ["added_a", "added_b"]);
        assert_eq!(diff.removed_cells, ["removed_a", "removed_b"]);
        assert_eq!(diff.modified_cells.len(), 1);
        assert_eq!(diff.modified_cells[0].cell_name, "modified");
        assert!(matches!(
            diff.modified_cells[0].elements.as_slice(),
            [
                ElementDiff::Modified { index: 1, .. },
                ElementDiff::Added { index: 2, .. }
            ]
        ));

        let reverse_diff = after.diff(&before, LibraryDiffOptions::default());
        assert!(matches!(
            reverse_diff.modified_cells[0].elements.as_slice(),
            [
                ElementDiff::Modified { index: 1, .. },
                ElementDiff::Removed { index: 2, .. }
            ]
        ));
    }

    #[test]
    fn coordinate_tolerance_controls_geometry_changes() {
        let mut before_cell = Cell::new("top");
        before_cell.add(polygon(0.0));
        let before = library_with_cell(before_cell);

        let mut after_cell = Cell::new("top");
        after_cell.add(polygon(0.0005));
        let after = library_with_cell(after_cell);

        assert!(
            before
                .diff(
                    &after,
                    LibraryDiffOptions {
                        coordinate_tolerance: 1e-9,
                    },
                )
                .is_empty()
        );
        assert!(
            !before
                .diff(&after, LibraryDiffOptions::default())
                .is_empty()
        );
    }

    #[test]
    fn coordinate_tolerance_applies_to_every_element_variant() {
        let point = |offset| Point::float(offset, offset, 1e-9);
        let path = |offset| {
            Path::new(
                [point(offset), point(10.0 + offset)],
                Layer::new(1),
                DataType::new(2),
                None,
                Some(Unit::float(2.0 + offset, 1e-9)),
                None,
                None,
            )
        };
        let text = |offset| {
            Text::new(
                "label",
                point(offset),
                Layer::new(3),
                DataType::new(4),
                1.0,
                Radians::new(0.0),
                false,
                VerticalPresentation::Middle,
                HorizontalPresentation::Centre,
            )
        };
        let reference =
            |offset| Reference::new("leaf").with_grid(Grid::default().with_origin(point(offset)));

        let before = [
            Element::Path(path(0.0)),
            Element::Polygon(polygon(0.0)),
            Element::Box(GdsBox::new(
                point(0.0),
                point(10.0),
                Layer::new(1),
                DataType::new(0),
            )),
            Element::Node(Node::new(vec![point(0.0)], Layer::new(1), DataType::new(0))),
            Element::Text(text(0.0)),
            Element::Reference(reference(0.0)),
        ];
        let after = [
            Element::Path(path(0.5)),
            Element::Polygon(polygon(0.0005)),
            Element::Box(GdsBox::new(
                point(0.5),
                point(10.5),
                Layer::new(1),
                DataType::new(0),
            )),
            Element::Node(Node::new(vec![point(0.5)], Layer::new(1), DataType::new(0))),
            Element::Text(text(0.5)),
            Element::Reference(reference(0.5)),
        ];

        assert!(
            before
                .iter()
                .zip(&after)
                .all(|(before, after)| elements_equal(before, after, 1e-9))
        );
    }

    #[test]
    fn non_coordinate_fields_remain_exact() {
        let before = Element::Text(Text::default());
        let after = Element::Text(Text::default().set_magnification(2.0));

        assert!(!elements_equal(&before, &after, f64::INFINITY));
    }

    #[test]
    fn nested_inline_reference_elements_use_coordinate_tolerance() {
        let before = Element::Reference(Reference::new(polygon(0.0)));
        let after = Element::Reference(Reference::new(polygon(0.0005)));

        assert!(elements_equal(&before, &after, 1e-9));
        assert!(!elements_equal(&before, &after, 0.0));
    }
}
