use std::collections::{BTreeMap, HashMap, HashSet};

use crate::cell::Cell;
use crate::library::Library;
use crate::types::{DataType, Layer};

/// Summary statistics for a single cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellStats {
    /// Number of polygon elements.
    pub polygon_count: usize,
    /// Number of path elements.
    pub path_count: usize,
    /// Number of box elements.
    pub box_count: usize,
    /// Number of text elements.
    pub text_count: usize,
    /// Number of reference elements.
    pub reference_count: usize,
    /// Element count per (layer, datatype) pair.
    pub elements_per_layer: BTreeMap<(Layer, DataType), usize>,
    /// Reference count per target cell name.
    pub references_per_cell: BTreeMap<String, usize>,
}

impl CellStats {
    /// Computes statistics for a cell.
    pub fn from_cell(cell: &Cell) -> Self {
        let polygon_count = cell.polygons().count();
        let path_count = cell.paths().count();
        let box_count = cell.boxes().count();
        let text_count = cell.texts().count();
        let reference_count = cell.references().count();

        let mut elements_per_layer: BTreeMap<(Layer, DataType), usize> = BTreeMap::new();

        for polygon in cell.polygons() {
            *elements_per_layer
                .entry((polygon.layer(), polygon.data_type()))
                .or_default() += 1;
        }

        for path in cell.paths() {
            *elements_per_layer
                .entry((path.layer(), path.data_type()))
                .or_default() += 1;
        }

        for gds_box in cell.boxes() {
            *elements_per_layer
                .entry((gds_box.layer(), gds_box.box_type()))
                .or_default() += 1;
        }

        for text in cell.texts() {
            *elements_per_layer
                .entry((text.layer(), text.data_type()))
                .or_default() += 1;
        }

        let mut references_per_cell: BTreeMap<String, usize> = BTreeMap::new();
        for name in cell.referenced_cell_names() {
            *references_per_cell.entry(name.to_string()).or_default() += 1;
        }

        Self {
            polygon_count,
            path_count,
            box_count,
            text_count,
            reference_count,
            elements_per_layer,
            references_per_cell,
        }
    }

    /// Total number of elements (polygons + paths + boxes + texts + references).
    pub fn total_elements(&self) -> usize {
        self.polygon_count
            + self.path_count
            + self.box_count
            + self.text_count
            + self.reference_count
    }
}

/// Summary statistics for an entire library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibraryStats {
    /// Number of cells in the library.
    pub cell_count: usize,
    /// Maximum hierarchy depth (0 for flat designs with no references).
    pub hierarchy_depth: usize,
    /// Per-cell statistics, keyed by cell name.
    pub cell_stats: BTreeMap<String, CellStats>,
}

impl LibraryStats {
    /// Computes statistics for a library.
    pub fn from_library(library: &Library) -> Self {
        let cell_stats: BTreeMap<String, CellStats> = library
            .cells()
            .iter()
            .map(|(name, cell)| (name.clone(), CellStats::from_cell(cell)))
            .collect();

        let hierarchy_depth = compute_hierarchy_depth(library);

        Self {
            cell_count: library.cells().len(),
            hierarchy_depth,
            cell_stats,
        }
    }
}

/// Computes the maximum hierarchy depth across all cells.
///
/// A cell with no references has depth 0. A cell referencing only leaf cells has
/// depth 1, and so on. Cycles are detected and do not cause infinite recursion.
fn compute_hierarchy_depth(library: &Library) -> usize {
    let mut cache: HashMap<&str, usize> = HashMap::new();
    let mut max_depth = 0;

    for name in library.cells().keys() {
        let depth = cell_depth(name, library, &mut cache, &mut HashSet::new());
        max_depth = max_depth.max(depth);
    }

    max_depth
}

fn cell_depth<'a>(
    name: &'a str,
    library: &'a Library,
    cache: &mut HashMap<&'a str, usize>,
    visiting: &mut HashSet<&'a str>,
) -> usize {
    if let Some(&depth) = cache.get(name) {
        return depth;
    }

    if !visiting.insert(name) {
        return 0;
    }

    let depth = if let Some(cell) = library.get_cell(name) {
        let mut max_child = 0;
        for target in cell.referenced_cell_names() {
            let child_depth = cell_depth(target, library, cache, visiting);
            max_child = max_child.max(child_depth + 1);
        }
        max_child
    } else {
        0
    };

    visiting.remove(name);
    cache.insert(name, depth);
    depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{Path, Polygon, Reference, Text};
    use crate::{GdsBox, Point};

    const UNITS: f64 = 1e-9;

    fn p(x: i32, y: i32) -> Point {
        Point::integer(x, y, UNITS)
    }

    #[test]
    fn empty_cell_stats() {
        let cell = Cell::new("empty");
        let stats = CellStats::from_cell(&cell);

        insta::assert_debug_snapshot!(stats, @r#"
        CellStats {
            polygon_count: 0,
            path_count: 0,
            box_count: 0,
            text_count: 0,
            reference_count: 0,
            elements_per_layer: {},
            references_per_cell: {},
        }
        "#);
        assert_eq!(stats.total_elements(), 0);
    }

    #[test]
    fn cell_stats_counts_elements() {
        let mut cell = Cell::new("mixed");
        cell.add(Polygon::new(
            [p(0, 0), p(10, 0), p(10, 10)],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Polygon::new(
            [p(0, 0), p(5, 0), p(5, 5), p(0, 5)],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Path::new(
            vec![p(0, 0), p(10, 10)],
            Layer::new(2),
            DataType::new(0),
            None,
            None,
            None,
            None,
        ));
        cell.add(GdsBox::new(
            p(0, 0),
            p(5, 5),
            Layer::new(3),
            DataType::new(1),
        ));
        cell.add(Text::default());
        cell.add(Reference::new("other".to_string()));

        let stats = CellStats::from_cell(&cell);

        insta::assert_debug_snapshot!(stats, @r#"
        CellStats {
            polygon_count: 2,
            path_count: 1,
            box_count: 1,
            text_count: 1,
            reference_count: 1,
            elements_per_layer: {
                (
                    Layer(
                        0,
                    ),
                    DataType(
                        0,
                    ),
                ): 1,
                (
                    Layer(
                        1,
                    ),
                    DataType(
                        0,
                    ),
                ): 2,
                (
                    Layer(
                        2,
                    ),
                    DataType(
                        0,
                    ),
                ): 1,
                (
                    Layer(
                        3,
                    ),
                    DataType(
                        1,
                    ),
                ): 1,
            },
            references_per_cell: {
                "other": 1,
            },
        }
        "#);
        assert_eq!(stats.total_elements(), 6);
    }

    #[test]
    fn empty_library_statistics() {
        let library = Library::new("empty");
        let stats = LibraryStats::from_library(&library);

        insta::assert_debug_snapshot!(stats, @r#"
        LibraryStats {
            cell_count: 0,
            hierarchy_depth: 0,
            cell_stats: {},
        }
        "#);
    }

    #[test]
    fn library_hierarchy_depth_flat() {
        let mut library = Library::new("flat");
        library.add_cell(Cell::new("a"));
        library.add_cell(Cell::new("b"));

        let stats = LibraryStats::from_library(&library);
        assert_eq!(stats.hierarchy_depth, 0);
    }

    #[test]
    fn library_hierarchy_depth_nested() {
        let mut library = Library::new("nested");

        let leaf = Cell::new("leaf");
        library.add_cell(leaf);

        let mut mid = Cell::new("mid");
        mid.add(Reference::new("leaf".to_string()));
        library.add_cell(mid);

        let mut top = Cell::new("top");
        top.add(Reference::new("mid".to_string()));
        library.add_cell(top);

        let stats = LibraryStats::from_library(&library);
        assert_eq!(stats.hierarchy_depth, 2);
        assert_eq!(stats.cell_count, 3);
    }

    #[test]
    fn library_hierarchy_depth_with_cycle() {
        let mut library = Library::new("cyclic");

        let mut a = Cell::new("a");
        a.add(Reference::new("b".to_string()));
        library.add_cell(a);

        let mut b = Cell::new("b");
        b.add(Reference::new("a".to_string()));
        library.add_cell(b);

        // Cycles are detected and do not cause infinite recursion.
        // The depth depends on processing order but is always finite.
        let stats = LibraryStats::from_library(&library);
        assert!(stats.hierarchy_depth <= 2);
    }

    #[test]
    fn library_hierarchy_depth_diamond() {
        let mut library = Library::new("diamond");

        library.add_cell(Cell::new("leaf"));

        let mut left = Cell::new("left");
        left.add(Reference::new("leaf".to_string()));
        library.add_cell(left);

        let mut right = Cell::new("right");
        right.add(Reference::new("leaf".to_string()));
        library.add_cell(right);

        let mut top = Cell::new("top");
        top.add(Reference::new("left".to_string()));
        top.add(Reference::new("right".to_string()));
        library.add_cell(top);

        let stats = LibraryStats::from_library(&library);
        assert_eq!(stats.hierarchy_depth, 2);
    }

    #[test]
    fn cell_stats_multiple_references_to_same_cell() {
        let mut cell = Cell::new("multi_ref");
        cell.add(Reference::new("target".to_string()));
        cell.add(Reference::new("target".to_string()));
        cell.add(Reference::new("other".to_string()));

        let stats = CellStats::from_cell(&cell);

        insta::assert_debug_snapshot!(stats.references_per_cell, @r#"
        {
            "other": 1,
            "target": 2,
        }
        "#);
    }
}
