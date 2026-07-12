use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::Write;

use crate::cell::Cell;
use crate::error::GdsError;
use crate::io::read::from_gds;
use crate::io::write::{GdsFileWriter, GdsWriter};
use crate::types::LayerMapping;
use crate::{Element, Instance};

/// A dangling reference: a cell contains a reference to a target that doesn't exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DanglingCellReference {
    /// The cell containing the dangling reference.
    pub cell_name: String,
    /// The name of the missing target cell.
    pub target_name: String,
}

/// A GDSII library containing named cells. This is the top-level container for a GDSII design.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    pub(crate) name: String,
    pub(crate) cells: HashMap<String, Cell>,
}

impl Library {
    /// Creates a new empty library with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
        }
    }

    /// Returns the library name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the map of cell names to cells.
    pub fn cells(&self) -> &HashMap<String, Cell> {
        &self.cells
    }

    /// Sets the library name.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Adds a cell to the library. If a cell with the same name exists, it is replaced.
    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.insert(cell.name().to_string(), cell);
    }

    /// Removes the given cells from the library by name.
    pub fn remove_cell(&mut self, cells: Vec<Cell>) {
        for cell in cells {
            self.cells.remove(cell.name());
        }
    }

    /// Returns a reference to the cell with the given name, if it exists.
    pub fn get_cell(&self, name: &str) -> Option<&Cell> {
        self.cells.get(name)
    }

    /// Returns a mutable reference to the cell with the given name, if it exists.
    pub fn get_cell_mut(&mut self, name: &str) -> Option<&mut Cell> {
        self.cells.get_mut(name)
    }

    /// Returns `true` if the library contains a cell with the same name.
    pub fn contains_cell(&self, cell: &Cell) -> bool {
        self.cells.contains_key(cell.name())
    }

    /// Serialize the library using a custom writer, returning the GDS bytes.
    pub fn write(
        &self,
        writer: &impl GdsWriter,
        user_units: f64,
        database_units: f64,
    ) -> Result<Vec<u8>, GdsError> {
        writer.write_library(self, user_units, database_units)
    }

    /// Write the library to a GDS file.
    ///
    /// The given user units are only used when writing the GDSII header.
    /// So, when you open the GDSII file you see values in these units.
    ///
    /// The database units are used when scaling values before writing.
    ///
    /// If you have a unit value 10 with units 1e-9, and database units are 1e-10,
    /// then the scaled value will be 100.
    pub fn write_file<P: AsRef<std::path::Path>>(
        &self,
        file_name: P,
        user_units: f64,
        database_units: f64,
    ) -> Result<(), GdsError> {
        let bytes = self.write(&GdsFileWriter, user_units, database_units)?;
        let mut file = File::create(file_name)?;
        file.write_all(&bytes)?;
        Ok(file.flush()?)
    }

    /// Detects and merges structurally identical cells.
    ///
    /// Two cells are considered identical if they have the same elements (ignoring cell name).
    /// When duplicates are found, the first cell (alphabetically) is kept as canonical,
    /// and all references to duplicates are updated to point to the canonical cell.
    ///
    /// Returns a map of removed cell names to the canonical cell name they were merged into.
    pub fn deduplicate_cells(&mut self) -> HashMap<String, String> {
        let mut names: Vec<String> = self.cells.keys().cloned().collect();
        names.sort();

        // Group cells with identical elements using pairwise comparison.
        // Each group is a vec of names sharing the same elements content.
        let mut groups: Vec<Vec<String>> = Vec::new();
        let mut assigned: std::collections::HashSet<String> = std::collections::HashSet::new();

        for i in 0..names.len() {
            if assigned.contains(&names[i]) {
                continue;
            }
            let mut group = vec![names[i].clone()];
            for name in names.iter().skip(i + 1) {
                if assigned.contains(name) {
                    continue;
                }
                if let (Some(cell_i), Some(cell_j)) =
                    (self.cells.get(&names[i]), self.cells.get(name))
                {
                    if cell_i.elements() == cell_j.elements() {
                        group.push(name.clone());
                        assigned.insert(name.clone());
                    }
                }
            }
            assigned.insert(names[i].clone());
            if group.len() > 1 {
                groups.push(group);
            }
        }

        // Build rename map: duplicate_name -> canonical_name (first alphabetically)
        let mut rename_map: HashMap<String, String> = HashMap::new();
        for group in &groups {
            let canonical = &group[0];
            for name in group.iter().skip(1) {
                rename_map.insert(name.clone(), canonical.clone());
            }
        }

        if rename_map.is_empty() {
            return rename_map;
        }

        // Remove duplicate cells
        for name in rename_map.keys() {
            self.cells.remove(name);
        }

        // Update all Instance::Cell references that point to removed duplicates
        for cell in self.cells.values_mut() {
            for element in cell.iter_elements_mut() {
                Self::update_reference_names(element, &rename_map);
            }
        }

        rename_map
    }

    /// Recursively updates `Instance::Cell` names in a reference element using the rename map.
    fn update_reference_names(element: &mut Element, rename_map: &HashMap<String, String>) {
        if let Element::Reference(reference) = element {
            match &mut reference.instance {
                Instance::Cell(name) => {
                    if let Some(canonical) = rename_map.get(name.as_str()) {
                        *name = canonical.clone();
                    }
                }
                Instance::Element(arc_elem) => {
                    let elem = std::sync::Arc::make_mut(arc_elem);
                    Self::update_reference_names(elem, rename_map);
                }
            }
        }
    }

    /// Remaps layer/data type pairs on all elements in all cells using the given mapping.
    pub fn remap_layers(&mut self, mapping: &LayerMapping) {
        for cell in self.cells.values_mut() {
            cell.remap_layers(mapping);
        }
    }

    /// Returns all dangling cell references in the library.
    ///
    /// A dangling cell reference is a `Reference` whose resolved cell name
    /// (via [`Reference::referenced_cell_name`]) does not match any cell in the library.
    /// This recursively resolves through inline element wrappers.
    pub fn dangling_cell_references(&self) -> Vec<DanglingCellReference> {
        let mut dangling = Vec::new();
        for (cell_name, cell) in &self.cells {
            for target in cell.referenced_cell_names() {
                if !self.cells.contains_key(target) {
                    dangling.push(DanglingCellReference {
                        cell_name: cell_name.clone(),
                        target_name: target.to_string(),
                    });
                }
            }
        }
        dangling
    }

    /// Builds an adjacency list of cell dependencies.
    ///
    /// For each cell in the library, collects the names of cells it directly references.
    pub fn dependency_graph(&self) -> HashMap<String, HashSet<String>> {
        self.cells
            .iter()
            .map(|(name, cell)| {
                let deps = cell
                    .referenced_cell_names()
                    .into_iter()
                    .map(String::from)
                    .collect();
                (name.clone(), deps)
            })
            .collect()
    }

    /// Returns all transitive dependencies of the given cell.
    ///
    /// Uses BFS to find every cell reachable from `cell_name` through the dependency graph.
    /// Returns an empty set if the cell does not exist.
    pub fn dependencies(&self, cell_name: &str) -> HashSet<String> {
        let graph = self.dependency_graph();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(direct) = graph.get(cell_name) {
            for dep in direct {
                if visited.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(deps) = graph.get(&current) {
                for dep in deps {
                    if visited.insert(dep.clone()) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        visited
    }

    /// Returns all cells that transitively depend on the given cell.
    ///
    /// Builds a reverse adjacency list and uses BFS to find every cell that
    /// directly or indirectly references `cell_name`.
    /// Returns an empty set if the cell does not exist.
    pub fn reverse_dependencies(&self, cell_name: &str) -> HashSet<String> {
        let graph = self.dependency_graph();

        let mut reverse: HashMap<String, HashSet<String>> = HashMap::new();
        for (cell, deps) in &graph {
            for dep in deps {
                reverse.entry(dep.clone()).or_default().insert(cell.clone());
            }
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        if let Some(dependents) = reverse.get(cell_name) {
            for dep in dependents {
                if visited.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }

        while let Some(current) = queue.pop_front() {
            if let Some(dependents) = reverse.get(&current) {
                for dep in dependents {
                    if visited.insert(dep.clone()) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        visited
    }

    /// Returns `true` if the dependency graph contains a cycle.
    ///
    /// Uses DFS with three-state coloring to detect back edges, which indicate
    /// circular references.
    pub fn has_circular_references(&self) -> bool {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            Unvisited,
            InProgress,
            Visited,
        }

        fn dfs<'a>(
            node: &'a str,
            graph: &'a HashMap<String, HashSet<String>>,
            states: &mut HashMap<&'a str, State>,
        ) -> bool {
            states.insert(node, State::InProgress);
            if let Some(deps) = graph.get(node) {
                for dep in deps {
                    match states.get(dep.as_str()).copied() {
                        Some(State::InProgress) => return true,
                        Some(State::Unvisited) if dfs(dep.as_str(), graph, states) => return true,
                        Some(State::Unvisited) => {}
                        _ => {}
                    }
                }
            }
            states.insert(node, State::Visited);
            false
        }

        let graph = self.dependency_graph();
        let mut states: HashMap<&str, State> = graph
            .keys()
            .map(|k| (k.as_str(), State::Unvisited))
            .collect();

        let keys: Vec<String> = graph.keys().cloned().collect();
        for key in &keys {
            if states.get(key.as_str()) == Some(&State::Unvisited)
                && dfs(key.as_str(), &graph, &mut states)
            {
                return true;
            }
        }

        false
    }

    /// Computes the hierarchy depth of the given cell.
    ///
    /// The depth is the length of the longest path from `cell_name` through its dependencies.
    /// A leaf cell (no references) has depth 0.
    /// Returns 0 if the cell does not exist or has no references.
    pub fn hierarchy_depth(&self, cell_name: &str) -> usize {
        fn depth_of(
            node: &str,
            graph: &HashMap<String, HashSet<String>>,
            cache: &mut HashMap<String, usize>,
            visiting: &mut HashSet<String>,
        ) -> usize {
            if let Some(&d) = cache.get(node) {
                return d;
            }
            if !visiting.insert(node.to_string()) {
                // Cycle detected; return 0 to avoid infinite recursion.
                return 0;
            }
            let d = graph
                .get(node)
                .map(|deps| {
                    deps.iter()
                        .map(|dep| 1 + depth_of(dep, graph, cache, visiting))
                        .max()
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            visiting.remove(node);
            cache.insert(node.to_string(), d);
            d
        }

        let graph = self.dependency_graph();
        let mut cache = HashMap::new();
        let mut visiting = HashSet::new();
        depth_of(cell_name, &graph, &mut cache, &mut visiting)
    }

    /// Read a library from a GDS file.
    ///
    /// The given units are not required they are used to normalize the database units
    /// to some more readable values. For example a unit value of 10 with units 1e-9
    /// and database units of 1e-10 will result in a scaled value of 100.
    /// This means you can work with the values in a more human-readable format.
    pub fn read_file<P: AsRef<std::path::Path>>(
        file_name: P,
        units: Option<f64>,
    ) -> Result<Self, GdsError> {
        from_gds(file_name, units)
    }
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library '{}' with {} cells", self.name, self.cells.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::elements::{Polygon, Reference};
    use crate::{DataType, Layer, Point};

    use super::*;

    #[test]
    fn test_library_new() {
        let mut library: Library = Library::new("test_lib");
        assert_eq!(library.name, "test_lib");
        assert!(library.cells.is_empty());

        library.set_name("new_name");
        assert_eq!(library.name, "new_name");
    }

    #[test]
    fn test_library_add_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");

        library.add_cell(cell.clone());
        assert_eq!(library.cells.len(), 1);
        assert!(library.cells.contains_key("test_cell"));
        assert_eq!(library.cells.get("test_cell"), Some(&cell));
    }

    #[test]
    fn test_library_get_cell_mut() {
        let mut library: Library = Library::new("test_lib");
        library.add_cell(Cell::new("test_cell"));

        let Some(cell) = library.get_cell_mut("test_cell") else {
            panic!("cell should exist");
        };
        cell.add(Polygon::default());

        assert_eq!(
            library
                .get_cell("test_cell")
                .expect("cell should exist")
                .elements()
                .len(),
            1
        );
    }

    #[test]
    fn test_library_add_multiple_cells() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1);
        library.add_cell(cell2);

        assert_eq!(library.cells.len(), 2);
        assert!(library.cells.contains_key("cell1"));
        assert!(library.cells.contains_key("cell2"));
    }

    #[test]
    fn test_library_add_duplicate_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("test_cell");
        let cell2 = Cell::new("test_cell");

        library.add_cell(cell1);
        library.add_cell(cell2.clone());

        assert_eq!(library.cells.len(), 1);
        assert_eq!(library.cells.get("test_cell"), Some(&cell2));
    }

    #[test]
    fn test_library_remove_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1.clone());
        library.add_cell(cell2);

        library.remove_cell(vec![cell1]);

        assert_eq!(library.cells.len(), 1);
        assert!(!library.cells.contains_key("cell1"));
        assert!(library.cells.contains_key("cell2"));
    }

    #[test]
    fn test_library_remove_nonexistent_cell() {
        let mut library: Library = Library::new("test_lib");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1);
        library.remove_cell(vec![cell2]);

        assert_eq!(library.cells.len(), 1);
        assert!(library.cells.contains_key("cell1"));
    }

    #[test]
    fn test_library_contains() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");
        let other_cell = Cell::new("other_cell");

        library.add_cell(cell.clone());

        assert!(library.contains_cell(&cell));
        assert!(!library.contains_cell(&other_cell));
    }

    #[test]
    fn test_library_display() {
        let mut library: Library = Library::new("my_library");
        let cell1 = Cell::new("cell1");
        let cell2 = Cell::new("cell2");

        library.add_cell(cell1);
        library.add_cell(cell2);

        insta::assert_snapshot!(library.to_string(), @"Library 'my_library' with 2 cells");
    }

    #[test]
    fn test_library_display_empty() {
        let library: Library = Library::new("empty_lib");
        insta::assert_snapshot!(library.to_string(), @"Library 'empty_lib' with 0 cells");
    }

    #[test]
    fn test_library_clone() {
        let mut library: Library = Library::new("test_lib");
        let cell = Cell::new("test_cell");
        library.add_cell(cell);

        let cloned = library.clone();
        assert_eq!(library, cloned);
        assert_eq!(library.name, cloned.name);
        assert_eq!(library.cells.len(), cloned.cells.len());
    }

    #[test]
    fn test_library_debug() {
        let library: Library = Library::new("debug_lib");
        insta::assert_snapshot!(format!("{library:?}"), @r#"Library { name: "debug_lib", cells: {} }"#);
    }

    #[test]
    fn test_remap_layers_across_library() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(crate::Path::new(
            vec![Point::integer(0, 0, units), Point::integer(5, 5, units)],
            Layer::new(2),
            DataType::new(3),
            None,
            None,
            None,
            None,
        ));
        library.add_cell(cell);

        let mapping: crate::LayerMapping = [
            (
                (Layer::new(1), DataType::new(0)),
                (Layer::new(10), DataType::new(20)),
            ),
            (
                (Layer::new(2), DataType::new(3)),
                (Layer::new(22), DataType::new(33)),
            ),
        ]
        .into_iter()
        .collect();

        library.remap_layers(&mapping);

        let cell = library.get_cell("cell").unwrap();
        let polygon = cell.polygons().next().unwrap();
        insta::assert_debug_snapshot!(
            (polygon.layer(), polygon.data_type()),
            @r#"
        (
            Layer(
                10,
            ),
            DataType(
                20,
            ),
        )
        "#
        );
        let path = cell.paths().next().unwrap();
        insta::assert_debug_snapshot!(
            (path.layer(), path.data_type()),
            @r#"
        (
            Layer(
                22,
            ),
            DataType(
                33,
            ),
        )
        "#
        );
    }

    #[test]
    fn test_remap_layers_inline_element() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let polygon = Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let mut cell = Cell::new("cell");
        cell.add(Reference::new(polygon));
        library.add_cell(cell);

        let mapping: crate::LayerMapping = std::iter::once((
            (Layer::new(1), DataType::new(0)),
            (Layer::new(50), DataType::new(60)),
        ))
        .collect();

        library.remap_layers(&mapping);

        let cell = library.get_cell("cell").unwrap();
        let reference = cell.references().next().unwrap();
        let inner = reference.instance().as_element().unwrap();
        let polygon = inner.as_polygon().unwrap();
        insta::assert_debug_snapshot!(
            (polygon.layer(), polygon.data_type()),
            @r#"
        (
            Layer(
                50,
            ),
            DataType(
                60,
            ),
        )
        "#
        );
    }

    #[test]
    fn test_remap_layers_unmatched_unchanged() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(5),
            DataType::new(6),
        ));
        library.add_cell(cell);

        let mapping: crate::LayerMapping = std::iter::once((
            (Layer::new(99), DataType::new(99)),
            (Layer::new(1), DataType::new(1)),
        ))
        .collect();

        library.remap_layers(&mapping);

        let cell = library.get_cell("cell").unwrap();
        let polygon = cell.polygons().next().unwrap();
        insta::assert_debug_snapshot!(
            (polygon.layer(), polygon.data_type()),
            @r#"
        (
            Layer(
                5,
            ),
            DataType(
                6,
            ),
        )
        "#
        );
    }

    #[test]
    fn test_dangling_cell_references_none() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut base = Cell::new("base");
        base.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        library.add_cell(base);

        let mut top = Cell::new("top");
        top.add(Reference::new("base".to_string()));
        library.add_cell(top);

        assert!(library.dangling_cell_references().is_empty());
    }

    #[test]
    fn test_dangling_cell_references_detected() {
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell_a");
        cell.add(Reference::new("missing_cell".to_string()));
        library.add_cell(cell);

        insta::assert_debug_snapshot!(library.dangling_cell_references(), @r#"
        [
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "missing_cell",
            },
        ]
        "#);
    }

    #[test]
    fn test_dangling_cell_references_multiple() {
        let mut library = Library::new("lib");

        let mut cell_a = Cell::new("cell_a");
        cell_a.add(Reference::new("ghost1".to_string()));
        cell_a.add(Reference::new("ghost2".to_string()));
        library.add_cell(cell_a);

        let mut cell_b = Cell::new("cell_b");
        cell_b.add(Reference::new("ghost3".to_string()));
        library.add_cell(cell_b);

        let mut dangling = library.dangling_cell_references();
        dangling
            .sort_by(|a, b| (&a.cell_name, &a.target_name).cmp(&(&b.cell_name, &b.target_name)));

        insta::assert_debug_snapshot!(dangling, @r#"
        [
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "ghost1",
            },
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "ghost2",
            },
            DanglingCellReference {
                cell_name: "cell_b",
                target_name: "ghost3",
            },
        ]
        "#);
    }

    #[test]
    fn test_dangling_cell_references_inline_element() {
        let units = 1e-9;
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell_a");
        cell.add(Reference::new(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(10, 0, units),
                Point::integer(10, 10, units),
            ],
            Layer::new(1),
            DataType::new(0),
        )));
        library.add_cell(cell);

        assert!(library.dangling_cell_references().is_empty());
    }

    #[test]
    fn test_dangling_cell_references_nested_inline_element() {
        let mut library = Library::new("lib");

        let mut cell = Cell::new("cell_a");
        let inner_ref = Reference::new("missing_cell".to_string());
        cell.add(Reference::new(inner_ref));
        library.add_cell(cell);

        insta::assert_debug_snapshot!(library.dangling_cell_references(), @r#"
        [
            DanglingCellReference {
                cell_name: "cell_a",
                target_name: "missing_cell",
            },
        ]
        "#);
    }

    #[test]
    fn test_deduplicate_identical_cells() {
        let units = 1e-9;
        let mut lib = Library::new("test");

        let mut cell_a = Cell::new("A");
        cell_a.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(1, 0, units),
                Point::integer(0, 1, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        let mut cell_b = Cell::new("B");
        cell_b.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(1, 0, units),
                Point::integer(0, 1, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        let mut cell_top = Cell::new("TOP");
        cell_top.add(Reference::new("A".to_string()));
        cell_top.add(Reference::new("B".to_string()));
        lib.add_cell(cell_a);
        lib.add_cell(cell_b);
        lib.add_cell(cell_top);

        let merged = lib.deduplicate_cells();

        insta::assert_debug_snapshot!(merged, @r#"
        {
            "B": "A",
        }
        "#);
        assert!(lib.get_cell("B").is_none());
        assert!(lib.get_cell("A").is_some());

        let top = lib.get_cell("TOP").expect("TOP cell should exist");
        let refs: Vec<&str> = top.referenced_cell_names().into_iter().collect();
        assert!(refs.iter().all(|&r| r == "A"));
        assert!(!refs.contains(&"B"));
    }

    #[test]
    fn test_deduplicate_no_duplicates() {
        let units = 1e-9;
        let mut lib = Library::new("test");

        let mut cell_a = Cell::new("A");
        cell_a.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(1, 0, units),
                Point::integer(0, 1, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        let mut cell_b = Cell::new("B");
        cell_b.add(Polygon::new(
            [
                Point::integer(5, 5, units),
                Point::integer(6, 5, units),
                Point::integer(5, 6, units),
            ],
            Layer::new(2),
            DataType::new(0),
        ));

        lib.add_cell(cell_a);
        lib.add_cell(cell_b);

        let merged = lib.deduplicate_cells();
        assert!(merged.is_empty());
        assert_eq!(lib.cells().len(), 2);
    }

    #[test]
    fn test_deduplicate_empty_library() {
        let mut lib = Library::new("empty");
        let merged = lib.deduplicate_cells();
        assert!(merged.is_empty());
        assert!(lib.cells().is_empty());
    }

    #[test]
    fn test_deduplicate_multiple_groups() {
        let units = 1e-9;
        let mut lib = Library::new("test");

        // Group 1: A and C have the same polygon
        let polygon1 = Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(1, 0, units),
                Point::integer(0, 1, units),
            ],
            Layer::new(1),
            DataType::new(0),
        );
        let mut cell_a = Cell::new("A");
        cell_a.add(polygon1.clone());
        let mut cell_c = Cell::new("C");
        cell_c.add(polygon1);

        // Group 2: B and D have a different polygon
        let polygon2 = Polygon::new(
            [
                Point::integer(10, 10, units),
                Point::integer(20, 10, units),
                Point::integer(10, 20, units),
            ],
            Layer::new(2),
            DataType::new(0),
        );
        let mut cell_b = Cell::new("B");
        cell_b.add(polygon2.clone());
        let mut cell_d = Cell::new("D");
        cell_d.add(polygon2);

        lib.add_cell(cell_a);
        lib.add_cell(cell_b);
        lib.add_cell(cell_c);
        lib.add_cell(cell_d);

        let merged = lib.deduplicate_cells();

        assert_eq!(merged.len(), 2);
        assert_eq!(merged.get("C"), Some(&"A".to_string()));
        assert_eq!(merged.get("D"), Some(&"B".to_string()));
        assert_eq!(lib.cells().len(), 2);
        assert!(lib.get_cell("A").is_some());
        assert!(lib.get_cell("B").is_some());
    }

    #[test]
    fn test_deduplicate_updates_nested_references() {
        let units = 1e-9;
        let mut lib = Library::new("test");

        let mut cell_a = Cell::new("A");
        cell_a.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(1, 0, units),
                Point::integer(0, 1, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        let mut cell_b = Cell::new("B");
        cell_b.add(Polygon::new(
            [
                Point::integer(0, 0, units),
                Point::integer(1, 0, units),
                Point::integer(0, 1, units),
            ],
            Layer::new(1),
            DataType::new(0),
        ));

        // Cell with a nested inline reference pointing to "B"
        let mut cell_top = Cell::new("TOP");
        let inner_ref = Reference::new("B".to_string());
        cell_top.add(Reference::new(inner_ref));
        lib.add_cell(cell_a);
        lib.add_cell(cell_b);
        lib.add_cell(cell_top);

        let merged = lib.deduplicate_cells();

        insta::assert_debug_snapshot!(merged, @r#"
        {
            "B": "A",
        }
        "#);

        let top = lib.get_cell("TOP").expect("TOP cell should exist");
        let refs: Vec<&str> = top.referenced_cell_names().into_iter().collect();
        assert_eq!(refs, vec!["A"]);
    }

    #[test]
    fn test_deduplicate_empty_cells_are_duplicates() {
        let mut lib = Library::new("test");
        lib.add_cell(Cell::new("X"));
        lib.add_cell(Cell::new("Y"));
        lib.add_cell(Cell::new("Z"));

        let merged = lib.deduplicate_cells();

        // All empty cells are identical; X is canonical (alphabetically first)
        assert_eq!(merged.len(), 2);
        assert_eq!(merged.get("Y"), Some(&"X".to_string()));
        assert_eq!(merged.get("Z"), Some(&"X".to_string()));
        assert_eq!(lib.cells().len(), 1);
        assert!(lib.get_cell("X").is_some());
    }

    /// Builds a test library with hierarchy: TOP -> {A, B}, A -> C, B and C are leaves.
    fn hierarchy_library() -> Library {
        let mut lib = Library::new("test");

        let mut top = Cell::new("TOP");
        top.add(Reference::new("A".to_string()));
        top.add(Reference::new("B".to_string()));

        let mut a = Cell::new("A");
        a.add(Reference::new("C".to_string()));

        let b = Cell::new("B");
        let c = Cell::new("C");

        lib.add_cell(top);
        lib.add_cell(a);
        lib.add_cell(b);
        lib.add_cell(c);
        lib
    }

    #[test]
    fn test_dependency_graph() {
        let lib = hierarchy_library();
        let graph = lib.dependency_graph();

        let mut top_deps: Vec<_> = graph["TOP"].iter().cloned().collect();
        top_deps.sort();
        insta::assert_debug_snapshot!(top_deps, @r#"
        [
            "A",
            "B",
        ]
        "#);

        insta::assert_debug_snapshot!(graph["A"], @r#"
        {
            "C",
        }
        "#);

        assert!(graph["B"].is_empty());
        assert!(graph["C"].is_empty());
    }

    #[test]
    fn test_dependencies_top() {
        let lib = hierarchy_library();
        let mut deps: Vec<_> = lib.dependencies("TOP").into_iter().collect();
        deps.sort();
        insta::assert_debug_snapshot!(deps, @r#"
        [
            "A",
            "B",
            "C",
        ]
        "#);
    }

    #[test]
    fn test_dependencies_mid() {
        let lib = hierarchy_library();
        insta::assert_debug_snapshot!(lib.dependencies("A"), @r#"
        {
            "C",
        }
        "#);
    }

    #[test]
    fn test_dependencies_leaf() {
        let lib = hierarchy_library();
        assert!(lib.dependencies("C").is_empty());
    }

    #[test]
    fn test_dependencies_nonexistent() {
        let lib = hierarchy_library();
        assert!(lib.dependencies("MISSING").is_empty());
    }

    #[test]
    fn test_reverse_dependencies_leaf() {
        let lib = hierarchy_library();
        let mut rev: Vec<_> = lib.reverse_dependencies("C").into_iter().collect();
        rev.sort();
        insta::assert_debug_snapshot!(rev, @r#"
        [
            "A",
            "TOP",
        ]
        "#);
    }

    #[test]
    fn test_reverse_dependencies_mid() {
        let lib = hierarchy_library();
        insta::assert_debug_snapshot!(lib.reverse_dependencies("A"), @r#"
        {
            "TOP",
        }
        "#);
    }

    #[test]
    fn test_reverse_dependencies_top() {
        let lib = hierarchy_library();
        assert!(lib.reverse_dependencies("TOP").is_empty());
    }

    #[test]
    fn test_reverse_dependencies_nonexistent() {
        let lib = hierarchy_library();
        assert!(lib.reverse_dependencies("MISSING").is_empty());
    }

    #[test]
    fn test_has_circular_references_false() {
        let lib = hierarchy_library();
        assert!(!lib.has_circular_references());
    }

    #[test]
    fn test_has_circular_references_true() {
        let mut lib = Library::new("circular");

        let mut a = Cell::new("A");
        a.add(Reference::new("B".to_string()));
        let mut b = Cell::new("B");
        b.add(Reference::new("A".to_string()));

        lib.add_cell(a);
        lib.add_cell(b);

        assert!(lib.has_circular_references());
    }

    #[test]
    fn test_has_circular_references_self() {
        let mut lib = Library::new("self_ref");

        let mut a = Cell::new("A");
        a.add(Reference::new("A".to_string()));

        lib.add_cell(a);

        assert!(lib.has_circular_references());
    }

    #[test]
    fn test_hierarchy_depth_top() {
        let lib = hierarchy_library();
        assert_eq!(lib.hierarchy_depth("TOP"), 2);
    }

    #[test]
    fn test_hierarchy_depth_mid() {
        let lib = hierarchy_library();
        assert_eq!(lib.hierarchy_depth("A"), 1);
    }

    #[test]
    fn test_hierarchy_depth_leaf() {
        let lib = hierarchy_library();
        assert_eq!(lib.hierarchy_depth("C"), 0);
        assert_eq!(lib.hierarchy_depth("B"), 0);
    }

    #[test]
    fn test_hierarchy_depth_nonexistent() {
        let lib = hierarchy_library();
        assert_eq!(lib.hierarchy_depth("MISSING"), 0);
    }

    #[test]
    fn test_hierarchy_depth_with_cycle() {
        let mut lib = Library::new("circular");

        let mut a = Cell::new("A");
        a.add(Reference::new("B".to_string()));
        let mut b = Cell::new("B");
        b.add(Reference::new("A".to_string()));

        lib.add_cell(a);
        lib.add_cell(b);

        // Cycles are handled gracefully without infinite recursion.
        let _ = lib.hierarchy_depth("A");
    }

    #[test]
    fn test_empty_library_graph() {
        let lib = Library::new("empty");
        assert!(lib.dependency_graph().is_empty());
        assert!(!lib.has_circular_references());
    }
}
