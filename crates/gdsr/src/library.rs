use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::Write;

use crate::cell::Cell;
use crate::design_rules::{self, DesignRuleOptions, DesignRuleReport};
use crate::error::GdsError;
use crate::io::read::{from_gds, from_gds_filtered};
use crate::io::write::validation::MAX_STRUCTURE_NAME_LENGTH;
use crate::io::write::{GdsFileWriter, GdsWriter};
use crate::types::LayerMapping;
use crate::{DataType, Element, GdsTimestampPolicy, GdsTimestamps, Instance, Layer};

/// A dangling reference: a cell contains a reference to a target that doesn't exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DanglingCellReference {
    /// The cell containing the dangling reference.
    pub cell_name: String,
    /// The name of the missing target cell.
    pub target_name: String,
}

/// Whether unreachable-cell pruning should modify the library.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellPruneMode {
    /// Return the report without removing cells.
    DryRun,
    /// Remove the cells listed in the report.
    Apply,
}

/// The result of unreachable-cell pruning.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CellPruneReport {
    /// Cell names removed, or that would be removed in dry-run mode, in stable name order.
    pub removed_cells: Vec<String>,
    /// Removed cells that were unreachable from every discovered top cell.
    pub globally_unreachable_cells: Vec<String>,
    /// Direct elements reclaimed from removed cells, without expanding references.
    pub reclaimed_element_count: usize,
}

/// An error produced while analyzing or pruning unreachable cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellPruneError {
    /// One or more retained roots do not exist in the library.
    UnknownRoots { cell_names: Vec<String> },
}

impl fmt::Display for CellPruneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRoots { cell_names } => {
                write!(
                    formatter,
                    "unknown retained root cells: {}",
                    cell_names.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for CellPruneError {}

/// How [`Library::merge`] handles an incoming cell whose name already exists.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellConflictStrategy {
    /// Reject the merge without modifying the primary library.
    #[default]
    Error,
    /// Give the incoming cell a unique numeric suffix and update its library's references.
    Rename,
    /// Replace the existing cell with the incoming cell.
    Overwrite,
    /// Keep the existing cell and discard the incoming cell.
    Skip,
}

/// An error produced while merging GDS libraries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LibraryMergeError {
    /// An incoming cell conflicts with an existing cell under
    /// [`CellConflictStrategy::Error`].
    CellNameConflict { cell_name: String },
    /// The merged cell set contains references to cells that do not exist.
    DanglingCellReferences {
        references: Vec<DanglingCellReference>,
    },
}

impl fmt::Display for LibraryMergeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CellNameConflict { cell_name } => {
                write!(formatter, "cell name conflict: {cell_name}")
            }
            Self::DanglingCellReferences { references } => write!(
                formatter,
                "merged library contains {} dangling cell references",
                references.len()
            ),
        }
    }
}

impl std::error::Error for LibraryMergeError {}

/// A GDSII library containing named cells. This is the top-level container for a GDSII design.
/// Timestamp metadata is excluded from equality comparisons.
#[derive(Clone, Debug)]
pub struct Library {
    pub(crate) name: String,
    pub(crate) cells: HashMap<String, Cell>,
    pub(crate) timestamps: Option<GdsTimestamps>,
}

impl PartialEq for Library {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.cells == other.cells
    }
}

impl Library {
    /// Creates a new empty library with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cells: HashMap::new(),
            timestamps: None,
        }
    }

    /// Returns the library name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parsed or explicitly assigned `BGNLIB` timestamps.
    pub const fn timestamps(&self) -> Option<GdsTimestamps> {
        self.timestamps
    }

    /// Assigns the last modification and access times used by Preserve writes.
    pub fn set_timestamps(&mut self, timestamps: GdsTimestamps) {
        self.timestamps = Some(timestamps);
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

    /// Renames a cell and updates references to that cell.
    pub fn rename_cell(&mut self, old_name: &str, new_name: &str) -> bool {
        if old_name == new_name {
            return self.cells.contains_key(old_name);
        }
        if !self.cells.contains_key(old_name) || self.cells.contains_key(new_name) {
            return false;
        }

        let Some(mut cell) = self.cells.remove(old_name) else {
            return false;
        };
        cell.set_name(new_name);
        self.cells.insert(new_name.to_string(), cell);

        let rename_map = HashMap::from([(old_name.to_string(), new_name.to_string())]);
        for cell in self.cells.values_mut() {
            for element in cell.iter_elements_mut() {
                Self::update_reference_names(element, &rename_map);
            }
        }

        true
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

    /// Merges cells from `libraries` into this primary library.
    ///
    /// The primary library's name and metadata are preserved. Incoming libraries are
    /// applied in iteration order. The primary library is not modified if a conflict or
    /// dangling reference causes the merge to fail.
    pub fn merge(
        &mut self,
        libraries: impl IntoIterator<Item = Self>,
        conflict_strategy: CellConflictStrategy,
    ) -> Result<(), LibraryMergeError> {
        let mut incoming_cells = HashMap::new();
        let mut occupied_names: HashSet<String> = self.cells.keys().cloned().collect();

        for library in libraries {
            let mut cell_names: Vec<String> = library.cells.keys().cloned().collect();
            cell_names.sort();
            let conflicting_names: Vec<String> = cell_names
                .iter()
                .filter(|name| occupied_names.contains(name.as_str()))
                .cloned()
                .collect();

            if conflict_strategy == CellConflictStrategy::Error
                && let Some(cell_name) = conflicting_names.first()
            {
                return Err(LibraryMergeError::CellNameConflict {
                    cell_name: cell_name.clone(),
                });
            }

            let rename_map = if conflict_strategy == CellConflictStrategy::Rename {
                let mut reserved_names = occupied_names.clone();
                reserved_names.extend(cell_names);
                conflicting_names
                    .into_iter()
                    .map(|cell_name| {
                        let renamed = Self::next_merged_cell_name(&cell_name, &mut reserved_names);
                        (cell_name, renamed)
                    })
                    .collect()
            } else {
                HashMap::new()
            };

            for (cell_name, mut cell) in library.cells {
                if conflict_strategy == CellConflictStrategy::Skip
                    && occupied_names.contains(&cell_name)
                {
                    continue;
                }

                if !rename_map.is_empty() {
                    for element in cell.iter_elements_mut() {
                        Self::update_reference_names(element, &rename_map);
                    }
                }

                let merged_name = rename_map.get(&cell_name).cloned().unwrap_or(cell_name);
                cell.set_name(&merged_name);
                occupied_names.insert(merged_name.clone());
                incoming_cells.insert(merged_name, cell);
            }
        }

        let mut dangling_references: Vec<DanglingCellReference> = self
            .cells
            .iter()
            .filter(|(cell_name, _)| !incoming_cells.contains_key(cell_name.as_str()))
            .chain(incoming_cells.iter())
            .flat_map(|(cell_name, cell)| {
                cell.referenced_cell_names()
                    .into_iter()
                    .filter(|target_name| !occupied_names.contains(*target_name))
                    .map(|target_name| DanglingCellReference {
                        cell_name: cell_name.clone(),
                        target_name: target_name.to_string(),
                    })
            })
            .collect();
        dangling_references.sort_by(|left, right| {
            (&left.cell_name, &left.target_name).cmp(&(&right.cell_name, &right.target_name))
        });

        if !dangling_references.is_empty() {
            return Err(LibraryMergeError::DanglingCellReferences {
                references: dangling_references,
            });
        }

        self.cells.extend(incoming_cells);
        Ok(())
    }

    fn next_merged_cell_name(cell_name: &str, reserved_names: &mut HashSet<String>) -> String {
        let mut index = 1_u64;
        loop {
            let suffix = format!("_{index}");
            let max_prefix_length = MAX_STRUCTURE_NAME_LENGTH.saturating_sub(suffix.len());
            let mut prefix_end = cell_name.len().min(max_prefix_length);
            while !cell_name.is_char_boundary(prefix_end) {
                prefix_end -= 1;
            }
            let candidate = format!("{}{suffix}", &cell_name[..prefix_end]);
            if reserved_names.insert(candidate.clone()) {
                return candidate;
            }
            index += 1;
        }
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

    /// Serializes the library with a custom writer and an explicit timestamp policy.
    pub fn write_with_timestamp_policy(
        &self,
        writer: &impl GdsWriter,
        user_units: f64,
        database_units: f64,
        timestamp_policy: GdsTimestampPolicy,
    ) -> Result<Vec<u8>, GdsError> {
        writer.write_library_with_timestamp_policy(
            self,
            user_units,
            database_units,
            timestamp_policy,
        )
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
        self.write_file_with_timestamp_policy(
            file_name,
            user_units,
            database_units,
            GdsTimestampPolicy::Current,
        )
    }

    /// Writes the library to a GDS file using an explicit timestamp policy.
    pub fn write_file_with_timestamp_policy<P: AsRef<std::path::Path>>(
        &self,
        file_name: P,
        user_units: f64,
        database_units: f64,
        timestamp_policy: GdsTimestampPolicy,
    ) -> Result<(), GdsError> {
        let bytes = self.write_with_timestamp_policy(
            &GdsFileWriter,
            user_units,
            database_units,
            timestamp_policy,
        )?;
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

    /// Returns cells that are not referenced by any cell, in stable name order.
    ///
    /// A component containing only circular references has no top cell.
    pub fn top_cells(&self) -> Vec<&Cell> {
        let referenced_cells: HashSet<&str> = self
            .cells
            .values()
            .flat_map(Cell::referenced_cell_names)
            .collect();
        let mut top_cells: Vec<&Cell> = self
            .cells
            .values()
            .filter(|cell| !referenced_cells.contains(cell.name()))
            .collect();
        top_cells.sort_by_key(|cell| cell.name());
        top_cells
    }

    /// Returns cells that cannot be reached from the retained roots, in stable name order.
    ///
    /// An empty `retained_roots` slice retains every discovered top cell, so the result then
    /// contains only components that are globally unreachable from the library's top cells.
    /// Dangling references are ignored. All explicit roots are validated before analysis.
    pub fn unreachable_cells(&self, retained_roots: &[&str]) -> Result<Vec<&Cell>, CellPruneError> {
        let unreachable = self.unreachable_cell_names(retained_roots)?;
        Ok(unreachable
            .iter()
            .filter_map(|name| self.cells.get(name))
            .collect())
    }

    /// Removes cells that cannot be reached from the retained roots.
    ///
    /// An empty `retained_roots` slice retains every discovered top cell. [`CellPruneMode::DryRun`]
    /// returns the same report as an applied prune without modifying the library. Unknown roots
    /// return an error before any mutation occurs.
    pub fn prune_unreachable(
        &mut self,
        retained_roots: &[&str],
        mode: CellPruneMode,
    ) -> Result<CellPruneReport, CellPruneError> {
        let removed_cells = self.unreachable_cell_names(retained_roots)?;
        let globally_unreachable = self.unreachable_cell_names(&[])?;
        let globally_unreachable_cells = removed_cells
            .iter()
            .filter(|name| globally_unreachable.binary_search(name).is_ok())
            .cloned()
            .collect();
        let reclaimed_element_count = removed_cells
            .iter()
            .filter_map(|name| self.cells.get(name))
            .map(|cell| cell.elements().len())
            .sum();

        let report = CellPruneReport {
            removed_cells,
            globally_unreachable_cells,
            reclaimed_element_count,
        };
        if mode == CellPruneMode::Apply {
            for cell_name in &report.removed_cells {
                self.cells.remove(cell_name);
            }
        }

        Ok(report)
    }

    fn unreachable_cell_names(
        &self,
        retained_roots: &[&str],
    ) -> Result<Vec<String>, CellPruneError> {
        let roots = if retained_roots.is_empty() {
            self.top_cells()
                .into_iter()
                .map(|cell| cell.name().to_string())
                .collect()
        } else {
            let mut roots: Vec<String> = retained_roots
                .iter()
                .map(|root| (*root).to_string())
                .collect();
            roots.sort();
            roots.dedup();

            let unknown_roots: Vec<String> = roots
                .iter()
                .filter(|root| !self.cells.contains_key(root.as_str()))
                .cloned()
                .collect();
            if !unknown_roots.is_empty() {
                return Err(CellPruneError::UnknownRoots {
                    cell_names: unknown_roots,
                });
            }
            roots
        };

        let mut reachable: HashSet<String> = roots.iter().cloned().collect();
        let mut queue = VecDeque::from(roots);
        while let Some(cell_name) = queue.pop_front() {
            if let Some(cell) = self.cells.get(&cell_name) {
                for target_name in cell.referenced_cell_names() {
                    if self.cells.contains_key(target_name)
                        && reachable.insert(target_name.to_string())
                    {
                        queue.push_back(target_name.to_string());
                    }
                }
            }
        }

        let mut unreachable: Vec<String> = self
            .cells
            .keys()
            .filter(|name| !reachable.contains(name.as_str()))
            .cloned()
            .collect();
        unreachable.sort();
        Ok(unreachable)
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

    /// Read a library from a GDS file, keeping only elements whose layer/data type
    /// pair matches the given filter.
    ///
    /// References are preserved because they do not carry layer/data type metadata.
    /// The given units behave the same way as [`Self::read_file`].
    pub fn read_file_filtered<P, F>(
        file_name: P,
        units: Option<f64>,
        layer_filter: F,
    ) -> Result<Self, GdsError>
    where
        P: AsRef<std::path::Path>,
        F: Fn(Layer, DataType) -> bool,
    {
        from_gds_filtered(file_name, units, layer_filter)
    }

    /// Run basic design-rule checks across all cells in the library.
    pub fn validate_design_rules(&self, options: &DesignRuleOptions) -> DesignRuleReport {
        design_rules::validate_library(self, options)
    }
}

impl std::fmt::Display for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Library '{}' with {} cells", self.name, self.cells.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::elements::{Path, Polygon, Reference, Text};
    use crate::{
        DEFAULT_INTEGER_UNITS, DataType, HorizontalPresentation, Layer, Point, Radians, Unit,
        VerticalPresentation,
    };

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
    fn read_file_filtered_keeps_only_matching_layer_data_type() {
        let dir = tempfile::tempdir().expect("temporary directory should be created");
        let path = dir.path().join("filtered.gds");

        let mut cell = Cell::new("top");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(10, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(10, 10, DEFAULT_INTEGER_UNITS),
                Point::integer(0, 10, DEFAULT_INTEGER_UNITS),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        cell.add(Path::new(
            [
                Point::integer(20, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(30, 0, DEFAULT_INTEGER_UNITS),
            ],
            Layer::new(2),
            DataType::new(0),
            None,
            Some(Unit::float(1.0, DEFAULT_INTEGER_UNITS)),
            None,
            None,
        ));

        let mut library = Library::new("test");
        library.add_cell(cell);
        library
            .write_file(&path, DEFAULT_INTEGER_UNITS, DEFAULT_INTEGER_UNITS)
            .expect("library should be writable");

        let read =
            Library::read_file_filtered(&path, Some(DEFAULT_INTEGER_UNITS), |layer, data| {
                layer == Layer::new(1) && data == DataType::new(0)
            })
            .expect("filtered library should be readable");
        let top = read.get_cell("top").expect("top cell should exist");

        assert_eq!(top.elements().len(), 1);
        assert!(matches!(top.elements()[0], Element::Polygon(_)));
    }

    #[test]
    fn read_file_filtered_uses_text_type() {
        let dir = tempfile::tempdir().expect("temporary directory should be created");
        let path = dir.path().join("filtered_text.gds");

        let mut cell = Cell::new("top");
        cell.add(Polygon::new(
            [
                Point::integer(0, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(10, 0, DEFAULT_INTEGER_UNITS),
                Point::integer(10, 10, DEFAULT_INTEGER_UNITS),
                Point::integer(0, 10, DEFAULT_INTEGER_UNITS),
            ],
            Layer::new(3),
            DataType::new(0),
        ));
        cell.add(Text::new(
            "label",
            Point::integer(20, 0, DEFAULT_INTEGER_UNITS),
            Layer::new(3),
            DataType::new(7),
            1.0,
            Radians::new(0.0),
            false,
            VerticalPresentation::default(),
            HorizontalPresentation::default(),
        ));

        let mut library = Library::new("test");
        library.add_cell(cell);
        library
            .write_file(&path, DEFAULT_INTEGER_UNITS, DEFAULT_INTEGER_UNITS)
            .expect("library should be writable");

        let read =
            Library::read_file_filtered(&path, Some(DEFAULT_INTEGER_UNITS), |layer, data| {
                layer == Layer::new(3) && data == DataType::new(7)
            })
            .expect("filtered library should be readable");
        let top = read.get_cell("top").expect("top cell should exist");

        assert_eq!(top.elements().len(), 1);
        let Element::Text(text) = &top.elements()[0] else {
            panic!("filtered element should be text");
        };
        assert_eq!(text.data_type(), DataType::new(7));
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
    fn test_library_rename_cell_updates_references() {
        let mut library: Library = Library::new("test_lib");
        let target = Cell::new("old");
        let mut parent = Cell::new("parent");
        parent.add(Reference::new("old"));
        library.add_cell(target);
        library.add_cell(parent);

        assert!(library.rename_cell("old", "new"));

        assert!(library.get_cell("old").is_none());
        assert!(library.get_cell("new").is_some());
        let refs: Vec<&str> = library
            .get_cell("parent")
            .expect("parent cell should exist")
            .referenced_cell_names();
        assert_eq!(refs, vec!["new"]);
    }

    #[test]
    fn test_library_rename_cell_rejects_missing_or_duplicate_names() {
        let mut library: Library = Library::new("test_lib");
        library.add_cell(Cell::new("a"));
        library.add_cell(Cell::new("b"));

        assert!(!library.rename_cell("missing", "c"));
        assert!(!library.rename_cell("a", "b"));
        assert!(library.rename_cell("a", "a"));
    }

    #[test]
    fn merge_error_is_atomic_on_cell_name_conflict() {
        let mut primary = Library::new("primary");
        primary.add_cell(Cell::new("shared"));

        let mut incoming = Library::new("incoming");
        incoming.add_cell(Cell::new("shared"));
        incoming.add_cell(Cell::new("unique"));

        let error = primary
            .merge([incoming], CellConflictStrategy::Error)
            .expect_err("conflicting merge should fail");

        assert_eq!(
            error,
            LibraryMergeError::CellNameConflict {
                cell_name: "shared".to_string(),
            }
        );
        assert_eq!(primary.name(), "primary");
        assert_eq!(primary.cells().len(), 1);
        assert!(primary.get_cell("unique").is_none());
    }

    #[test]
    fn merge_rename_keeps_conflicting_cells_and_updates_their_references() {
        let mut primary = Library::new("primary");
        primary.add_cell(Cell::new("shared"));

        let mut first = Library::new("first");
        first.add_cell(Cell::new("shared"));
        let mut first_top = Cell::new("first_top");
        first_top.add(Reference::new("shared"));
        first.add_cell(first_top);

        let mut second = Library::new("second");
        second.add_cell(Cell::new("shared"));
        let mut second_top = Cell::new("second_top");
        second_top.add(Reference::new("shared"));
        second.add_cell(second_top);

        primary
            .merge([first, second], CellConflictStrategy::Rename)
            .expect("conflicting cells should be renamed");

        assert!(primary.get_cell("shared").is_some());
        assert!(primary.get_cell("shared_1").is_some());
        assert!(primary.get_cell("shared_2").is_some());
        assert_eq!(
            primary
                .get_cell("first_top")
                .expect("first top cell should exist")
                .referenced_cell_names(),
            vec!["shared_1"]
        );
        assert_eq!(
            primary
                .get_cell("second_top")
                .expect("second top cell should exist")
                .referenced_cell_names(),
            vec!["shared_2"]
        );
    }

    #[test]
    fn merge_overwrite_replaces_conflicts_and_skip_preserves_them() {
        let mut incoming_cell = Cell::new("shared");
        incoming_cell.add(Polygon::default());
        let mut incoming = Library::new("incoming");
        incoming.add_cell(incoming_cell);

        let mut overwritten = Library::new("primary");
        overwritten.add_cell(Cell::new("shared"));
        overwritten
            .merge([incoming.clone()], CellConflictStrategy::Overwrite)
            .expect("overwrite merge should succeed");

        let mut skipped = Library::new("primary");
        skipped.add_cell(Cell::new("shared"));
        skipped
            .merge([incoming], CellConflictStrategy::Skip)
            .expect("skip merge should succeed");

        assert_eq!(
            overwritten
                .get_cell("shared")
                .expect("overwritten cell should exist")
                .elements()
                .len(),
            1
        );
        assert!(
            skipped
                .get_cell("shared")
                .expect("preserved cell should exist")
                .elements()
                .is_empty()
        );
    }

    #[test]
    fn merge_resolves_references_across_incoming_libraries() {
        let mut referencing = Library::new("referencing");
        let mut top = Cell::new("top");
        top.add(Reference::new("leaf"));
        referencing.add_cell(top);

        let mut dependency = Library::new("dependency");
        dependency.add_cell(Cell::new("leaf"));

        let mut primary = Library::new("primary");
        primary
            .merge([referencing, dependency], CellConflictStrategy::Error)
            .expect("cross-library reference should resolve after the full merge");

        assert!(primary.dangling_cell_references().is_empty());
    }

    #[test]
    fn merge_rejects_dangling_references_without_modifying_primary() {
        let mut incoming = Library::new("incoming");
        let mut top = Cell::new("top");
        top.add(Reference::new("missing"));
        incoming.add_cell(top);

        let mut primary = Library::new("primary");
        let error = primary
            .merge([incoming], CellConflictStrategy::Error)
            .expect_err("dangling reference should fail the merge");

        assert_eq!(
            error,
            LibraryMergeError::DanglingCellReferences {
                references: vec![DanglingCellReference {
                    cell_name: "top".to_string(),
                    target_name: "missing".to_string(),
                }],
            }
        );
        assert!(primary.cells().is_empty());
    }

    #[test]
    fn merge_rename_keeps_names_within_the_gds_limit() {
        let original_name = "a".repeat(MAX_STRUCTURE_NAME_LENGTH);
        let mut primary = Library::new("primary");
        primary.add_cell(Cell::new(&original_name));

        let mut incoming = Library::new("incoming");
        incoming.add_cell(Cell::new(&original_name));
        primary
            .merge([incoming], CellConflictStrategy::Rename)
            .expect("conflicting cell should be renamed");

        let renamed_name = format!("{}_1", "a".repeat(MAX_STRUCTURE_NAME_LENGTH - 2));
        assert!(primary.get_cell(&renamed_name).is_some());
        assert_eq!(renamed_name.len(), MAX_STRUCTURE_NAME_LENGTH);
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
        insta::assert_snapshot!(format!("{library:?}"), @r#"Library { name: "debug_lib", cells: {}, timestamps: None }"#);
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

    fn prunable_library() -> Library {
        let mut library = Library::new("prunable");

        let mut keep_a = Cell::new("KEEP_A");
        keep_a.add(Reference::new("SHARED"));
        let mut keep_b = Cell::new("KEEP_B");
        keep_b.add(Reference::new("B_ONLY"));
        let mut drop = Cell::new("DROP");
        drop.add(Reference::new("SHARED"));
        drop.add(Reference::new("DROP_ONLY"));
        let mut drop_only = Cell::new("DROP_ONLY");
        drop_only.add(Polygon::default());

        library.add_cell(drop_only);
        library.add_cell(Cell::new("SHARED"));
        library.add_cell(keep_b);
        library.add_cell(drop);
        library.add_cell(Cell::new("B_ONLY"));
        library.add_cell(keep_a);
        library
    }

    #[test]
    fn top_cells_are_sorted_and_ignore_dangling_targets() {
        let mut library = Library::new("tops");
        let mut z_top = Cell::new("Z_TOP");
        z_top.add(Reference::new("SHARED"));
        z_top.add(Reference::new("MISSING"));
        let mut a_top = Cell::new("A_TOP");
        a_top.add(Reference::new("SHARED"));

        library.add_cell(z_top);
        library.add_cell(Cell::new("SHARED"));
        library.add_cell(a_top);

        let names: Vec<&str> = library.top_cells().into_iter().map(Cell::name).collect();
        assert_eq!(names, ["A_TOP", "Z_TOP"]);
        assert_eq!(library.dangling_cell_references().len(), 1);
    }

    #[test]
    fn unreachable_cells_preserve_multiple_roots_and_shared_descendants() {
        let library = prunable_library();

        let names: Vec<&str> = library
            .unreachable_cells(&["KEEP_B", "KEEP_A"])
            .expect("retained roots should exist")
            .into_iter()
            .map(Cell::name)
            .collect();

        assert_eq!(names, ["DROP", "DROP_ONLY"]);
    }

    #[test]
    fn dry_run_reports_sorted_cells_and_direct_element_count_without_mutating() {
        let mut library = prunable_library();
        let original = library.clone();

        let report = library
            .prune_unreachable(&["KEEP_A", "KEEP_B"], CellPruneMode::DryRun)
            .expect("retained roots should exist");

        assert_eq!(
            report,
            CellPruneReport {
                removed_cells: vec!["DROP".to_string(), "DROP_ONLY".to_string()],
                globally_unreachable_cells: Vec::new(),
                reclaimed_element_count: 3,
            }
        );
        assert_eq!(library, original);

        let applied = library
            .prune_unreachable(&["KEEP_A", "KEEP_B"], CellPruneMode::Apply)
            .expect("retained roots should exist");
        assert_eq!(applied, report);
        assert!(library.get_cell("DROP").is_none());
        assert!(library.get_cell("DROP_ONLY").is_none());
        assert!(library.get_cell("SHARED").is_some());
        assert!(library.get_cell("B_ONLY").is_some());
    }

    #[test]
    fn report_distinguishes_globally_unreachable_cycles() {
        let mut library = prunable_library();
        let mut cycle_a = Cell::new("CYCLE_A");
        cycle_a.add(Reference::new("CYCLE_B"));
        let mut cycle_b = Cell::new("CYCLE_B");
        cycle_b.add(Reference::new("CYCLE_A"));
        library.add_cell(cycle_b);
        library.add_cell(cycle_a);

        let report = library
            .prune_unreachable(&["KEEP_A", "KEEP_B"], CellPruneMode::DryRun)
            .expect("retained roots should exist");

        assert_eq!(
            report.removed_cells,
            ["CYCLE_A", "CYCLE_B", "DROP", "DROP_ONLY"]
        );
        assert_eq!(report.globally_unreachable_cells, ["CYCLE_A", "CYCLE_B"]);
        assert_eq!(report.reclaimed_element_count, 5);
    }

    #[test]
    fn empty_roots_prune_only_globally_unreachable_cycles() {
        let mut library = Library::new("cycle");
        let mut cycle_a = Cell::new("A");
        cycle_a.add(Reference::new("B"));
        let mut cycle_b = Cell::new("B");
        cycle_b.add(Reference::new("A"));
        library.add_cell(cycle_a);
        library.add_cell(cycle_b);

        let unreachable: Vec<&str> = library
            .unreachable_cells(&[])
            .expect("discovered roots are always valid")
            .into_iter()
            .map(Cell::name)
            .collect();
        assert_eq!(unreachable, ["A", "B"]);

        let explicitly_reachable = library
            .unreachable_cells(&["A"])
            .expect("explicit cycle root should exist");
        assert!(explicitly_reachable.is_empty());

        let report = library
            .prune_unreachable(&[], CellPruneMode::Apply)
            .expect("discovered roots are always valid");
        assert_eq!(report.removed_cells, ["A", "B"]);
        assert!(library.cells().is_empty());
    }

    #[test]
    fn unknown_roots_are_sorted_and_do_not_mutate() {
        let mut library = prunable_library();
        let original = library.clone();

        let error = library
            .prune_unreachable(
                &["UNKNOWN_Z", "UNKNOWN_A", "UNKNOWN_Z"],
                CellPruneMode::Apply,
            )
            .expect_err("unknown roots should fail");

        assert_eq!(
            error,
            CellPruneError::UnknownRoots {
                cell_names: vec!["UNKNOWN_A".to_string(), "UNKNOWN_Z".to_string()],
            }
        );
        assert_eq!(library, original);
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
