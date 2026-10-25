use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

use crate::{Element, Grid, Instance, Library, Point, Reference};

/// Axis-aligned bounds in physical world units.
///
/// Coordinates are absolute physical distances: a point value multiplied by
/// its stored unit scale. They are independent of any GDS output database unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldBounds {
    /// Minimum physical x coordinate.
    pub min_x: f64,
    /// Minimum physical y coordinate.
    pub min_y: f64,
    /// Maximum physical x coordinate.
    pub max_x: f64,
    /// Maximum physical y coordinate.
    pub max_y: f64,
}

impl WorldBounds {
    /// Creates bounds from minimum and maximum coordinates.
    pub const fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Returns whether these bounds overlap `other`.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
    }

    /// Returns the smallest bounds containing `self` and `other`.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min_x: self.min_x.min(other.min_x),
            min_y: self.min_y.min(other.min_y),
            max_x: self.max_x.max(other.max_x),
            max_y: self.max_y.max(other.max_y),
        }
    }

    /// Applies one reference grid and returns aggregate bounds for all its members.
    pub fn transformed_by_grid(&self, grid: &Grid) -> Option<Self> {
        transform_bounds_by_grid(Some(*self), grid).ok().flatten()
    }

    fn corners(&self) -> [(f64, f64); 4] {
        [
            (self.min_x, self.min_y),
            (self.max_x, self.min_y),
            (self.max_x, self.max_y),
            (self.min_x, self.max_y),
        ]
    }

    const fn is_finite(&self) -> bool {
        self.min_x.is_finite()
            && self.min_y.is_finite()
            && self.max_x.is_finite()
            && self.max_y.is_finite()
    }
}

/// Stable location of one top-level reference element.
///
/// One location represents the aggregate placement of the whole reference,
/// including every AREF member. Members are not expanded into separate entries.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceLocation {
    /// Name of the cell containing the reference element.
    pub cell_name: String,
    /// Stable index of the reference in that cell's element sequence.
    pub element_index: usize,
}

/// Non-fatal hierarchy problem found while calculating bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HierarchyBoundsDiagnostic {
    /// A reference targets a cell absent from the library.
    DanglingReference {
        /// Cell containing the reference element.
        source_cell: String,
        /// Index of the reference in the source cell.
        element_index: usize,
        /// Missing target cell name.
        target_cell: String,
    },
    /// One strongly connected component containing hierarchy cycles.
    Cycle {
        /// Sorted cells in the cyclic component.
        cells: Vec<String>,
        /// Cell containing the representative intra-component reference.
        source_cell: String,
        /// Index of the representative reference in the source cell.
        element_index: usize,
        /// Target of the representative reference.
        target_cell: String,
    },
    /// An element contains non-finite coordinates or transform values.
    NonFiniteElement {
        /// Cell containing the invalid top-level element.
        source_cell: String,
        /// Index of the invalid element in the source cell.
        element_index: usize,
    },
}

/// Fatal hierarchy-bounds query error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HierarchyBoundsError {
    /// Requested root cell does not exist.
    UnknownRoot { cell_name: String },
}

impl fmt::Display for HierarchyBoundsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRoot { cell_name } => {
                write!(formatter, "unknown hierarchy root cell '{cell_name}'")
            }
        }
    }
}

impl std::error::Error for HierarchyBoundsError {}

/// Detached bounds snapshot for the hierarchy reachable from one root cell.
///
/// Every query builds a fresh bottom-up memo. Later library mutations require
/// no invalidation: a later query recomputes from current library contents.
#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyBoundsReport {
    root_cell: String,
    root_bounds: Option<WorldBounds>,
    cell_bounds: BTreeMap<String, Option<WorldBounds>>,
    reference_bounds: BTreeMap<ReferenceLocation, Option<WorldBounds>>,
    diagnostics: Vec<HierarchyBoundsDiagnostic>,
}

impl HierarchyBoundsReport {
    /// Returns the queried root cell name.
    pub fn root_cell(&self) -> &str {
        &self.root_cell
    }

    /// Returns aggregate bounds in the root cell's local coordinate space.
    pub const fn root_bounds(&self) -> Option<WorldBounds> {
        self.root_bounds
    }

    /// Returns local-coordinate bounds for every reachable cell, sorted by name.
    pub const fn cell_bounds(&self) -> &BTreeMap<String, Option<WorldBounds>> {
        &self.cell_bounds
    }

    /// Returns aggregate bounds for every reachable top-level reference element.
    ///
    /// Each value uses its containing cell's local coordinate space.
    pub const fn reference_bounds(&self) -> &BTreeMap<ReferenceLocation, Option<WorldBounds>> {
        &self.reference_bounds
    }

    /// Returns non-fatal problems in deterministic canonical order.
    ///
    /// Dangling references come first, then cyclic components, then non-finite
    /// elements. Each group is sorted by its structured fields.
    pub fn diagnostics(&self) -> &[HierarchyBoundsDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Default)]
struct BoundsState {
    cell_bounds: BTreeMap<String, Option<WorldBounds>>,
    reference_bounds: BTreeMap<ReferenceLocation, Option<WorldBounds>>,
    non_finite_elements: BTreeSet<(String, usize)>,
    #[cfg(test)]
    cell_scan_counts: BTreeMap<String, usize>,
}

#[derive(Clone)]
struct GraphEdge {
    element_index: usize,
    target_cell: String,
}

struct ReachableGraph {
    edges: BTreeMap<String, Vec<GraphEdge>>,
    dangling_diagnostics: Vec<HierarchyBoundsDiagnostic>,
}

struct CycleComponent {
    cells: Vec<String>,
    source_cell: String,
    element_index: usize,
    target_cell: String,
}

struct SccAnalysis {
    cyclic_component_by_cell: HashMap<String, usize>,
    cycles: Vec<CycleComponent>,
}

struct TraversalFrame {
    cell_name: String,
    next_element_index: usize,
    bounds: Option<WorldBounds>,
}

impl TraversalFrame {
    fn new(cell_name: String) -> Self {
        Self {
            cell_name,
            next_element_index: 0,
            bounds: None,
        }
    }
}

enum TraversalAction {
    Complete,
    Direct(BoundsResult),
    Reference(ReferenceLocation, BoundsResult),
    Descend(String),
}

enum ReferenceEvaluation {
    Ready(BoundsResult),
    NeedCell(String),
}

struct ReferenceContext<'a> {
    source_cell: &'a str,
    scc_analysis: &'a SccAnalysis,
}

#[derive(Clone, Copy)]
struct NonFiniteBounds;

type BoundsResult = Result<Option<WorldBounds>, NonFiniteBounds>;

impl Library {
    /// Calculates hierarchy-aware bounds with one ephemeral bottom-up memo.
    ///
    /// Dangling references and cycles are reported, so valid sibling geometry
    /// remains. Cyclic cell bounds conservatively cover one finite traversal
    /// through their strongly connected component. Only an unknown root is fatal.
    /// Cell traversal uses heap-backed frames rather than the process call stack.
    pub fn hierarchy_bounds(
        &self,
        root_cell: &str,
    ) -> Result<HierarchyBoundsReport, HierarchyBoundsError> {
        let (state, diagnostics) = calculate_bounds(self, root_cell)?;
        let root_bounds = state.cell_bounds.get(root_cell).copied().flatten();
        Ok(HierarchyBoundsReport {
            root_cell: root_cell.to_string(),
            root_bounds,
            cell_bounds: state.cell_bounds,
            reference_bounds: state.reference_bounds,
            diagnostics,
        })
    }
}

fn calculate_bounds(
    library: &Library,
    root_cell: &str,
) -> Result<(BoundsState, Vec<HierarchyBoundsDiagnostic>), HierarchyBoundsError> {
    if library.get_cell(root_cell).is_none() {
        return Err(HierarchyBoundsError::UnknownRoot {
            cell_name: root_cell.to_string(),
        });
    }

    let graph = reachable_graph(library, root_cell);
    let scc_analysis = analyze_sccs(&graph);
    let mut state = BoundsState::default();
    calculate_cell_bounds(library, root_cell, &scc_analysis, &mut state)?;
    for cell_name in graph.edges.keys() {
        if !state.cell_bounds.contains_key(cell_name) {
            calculate_cell_bounds(library, cell_name, &scc_analysis, &mut state)?;
        }
    }
    if !scc_analysis.cycles.is_empty() {
        expand_cyclic_component_bounds(library, &scc_analysis, &mut state)?;
        let cyclic_cells = &scc_analysis.cyclic_component_by_cell;
        state
            .cell_bounds
            .retain(|cell_name, _| cyclic_cells.contains_key(cell_name));
        state
            .reference_bounds
            .retain(|location, _| cyclic_cells.contains_key(&location.cell_name));
        state
            .non_finite_elements
            .retain(|(cell_name, _)| cyclic_cells.contains_key(cell_name));

        if !state.cell_bounds.contains_key(root_cell) {
            calculate_cell_bounds(library, root_cell, &scc_analysis, &mut state)?;
        }
        for cell_name in graph.edges.keys() {
            if !state.cell_bounds.contains_key(cell_name) {
                calculate_cell_bounds(library, cell_name, &scc_analysis, &mut state)?;
            }
        }
    }

    let mut diagnostics = graph.dangling_diagnostics;
    diagnostics.extend(scc_analysis.cycles.into_iter().map(|cycle| {
        HierarchyBoundsDiagnostic::Cycle {
            cells: cycle.cells,
            source_cell: cycle.source_cell,
            element_index: cycle.element_index,
            target_cell: cycle.target_cell,
        }
    }));
    diagnostics.extend(
        state
            .non_finite_elements
            .iter()
            .map(
                |(source_cell, element_index)| HierarchyBoundsDiagnostic::NonFiniteElement {
                    source_cell: source_cell.clone(),
                    element_index: *element_index,
                },
            ),
    );

    Ok((state, diagnostics))
}

fn expand_cyclic_component_bounds(
    library: &Library,
    scc_analysis: &SccAnalysis,
    state: &mut BoundsState,
) -> Result<(), HierarchyBoundsError> {
    let acyclic_analysis = SccAnalysis {
        cyclic_component_by_cell: HashMap::new(),
        cycles: Vec::new(),
    };
    for component in cyclic_components_bottom_up(library, scc_analysis) {
        for cell_name in &component.cells {
            state.cell_bounds.remove(cell_name);
        }
        state
            .reference_bounds
            .retain(|location, _| component.cells.binary_search(&location.cell_name).is_err());
        state
            .non_finite_elements
            .retain(|(cell_name, _)| component.cells.binary_search(cell_name).is_err());
        for cell_name in &component.cells {
            calculate_cell_bounds(library, cell_name, scc_analysis, state)?;
        }

        for _ in 1..component.cells.len() {
            let previous: BTreeMap<_, _> = component
                .cells
                .iter()
                .filter_map(|cell_name| {
                    state
                        .cell_bounds
                        .get(cell_name)
                        .copied()
                        .map(|bounds| (cell_name.clone(), bounds))
                })
                .collect();
            let mut expanded = previous.clone();

            for source_cell in &component.cells {
                let Some(cell) = library.get_cell(source_cell) else {
                    continue;
                };
                for (element_index, element) in cell.elements().iter().enumerate() {
                    let Element::Reference(reference) = element else {
                        continue;
                    };
                    let Some(target_cell) = terminal_cell_name(reference) else {
                        continue;
                    };
                    if !component
                        .cells
                        .iter()
                        .any(|cell_name| cell_name == target_cell)
                    {
                        continue;
                    }
                    let evaluation = evaluate_reference(
                        library,
                        reference,
                        &ReferenceContext {
                            source_cell,
                            scc_analysis: &acyclic_analysis,
                        },
                        state,
                    );
                    match evaluation {
                        ReferenceEvaluation::Ready(Ok(Some(bounds))) => {
                            let current = expanded.entry(source_cell.clone()).or_default();
                            merge_bounds(current, bounds);
                        }
                        ReferenceEvaluation::Ready(Err(_)) => {
                            state
                                .non_finite_elements
                                .insert((source_cell.clone(), element_index));
                        }
                        ReferenceEvaluation::Ready(Ok(None)) | ReferenceEvaluation::NeedCell(_) => {
                        }
                    }
                }
            }

            for (cell_name, bounds) in expanded {
                state.cell_bounds.insert(cell_name, bounds);
            }
        }
    }

    for component in &scc_analysis.cycles {
        for source_cell in &component.cells {
            let Some(cell) = library.get_cell(source_cell) else {
                continue;
            };
            for (element_index, element) in cell.elements().iter().enumerate() {
                let Element::Reference(reference) = element else {
                    continue;
                };
                let Some(target_cell) = terminal_cell_name(reference) else {
                    continue;
                };
                if !component
                    .cells
                    .iter()
                    .any(|cell_name| cell_name == target_cell)
                {
                    continue;
                }
                let location = ReferenceLocation {
                    cell_name: source_cell.clone(),
                    element_index,
                };
                match evaluate_reference(
                    library,
                    reference,
                    &ReferenceContext {
                        source_cell,
                        scc_analysis: &acyclic_analysis,
                    },
                    state,
                ) {
                    ReferenceEvaluation::Ready(Ok(bounds)) => {
                        state.reference_bounds.insert(location, bounds);
                    }
                    ReferenceEvaluation::Ready(Err(_)) => {
                        state.reference_bounds.insert(location, None);
                        state
                            .non_finite_elements
                            .insert((source_cell.clone(), element_index));
                    }
                    ReferenceEvaluation::NeedCell(_) => {}
                }
            }
        }
    }

    Ok(())
}

fn cyclic_components_bottom_up<'a>(
    library: &Library,
    scc_analysis: &'a SccAnalysis,
) -> Vec<&'a CycleComponent> {
    let component_name_by_cell: HashMap<&str, &str> = scc_analysis
        .cycles
        .iter()
        .filter_map(|component| {
            component.cells.first().map(|component_name| {
                component
                    .cells
                    .iter()
                    .map(move |cell_name| (cell_name.as_str(), component_name.as_str()))
            })
        })
        .flatten()
        .collect();
    let component_by_name: HashMap<&str, &CycleComponent> = scc_analysis
        .cycles
        .iter()
        .filter_map(|component| {
            component
                .cells
                .first()
                .map(|name| (name.as_str(), component))
        })
        .collect();
    let mut adjacency: BTreeMap<String, Vec<String>> = component_by_name
        .keys()
        .map(|name| ((*name).to_string(), Vec::new()))
        .collect();

    for component in &scc_analysis.cycles {
        let Some(component_name) = component.cells.first() else {
            continue;
        };
        let Some(dependencies) = adjacency.get_mut(component_name) else {
            continue;
        };
        for cell_name in &component.cells {
            let Some(cell) = library.get_cell(cell_name) else {
                continue;
            };
            for reference in cell.elements().iter().filter_map(|element| {
                if let Element::Reference(reference) = element {
                    Some(reference)
                } else {
                    None
                }
            }) {
                let Some(target_cell) = terminal_cell_name(reference) else {
                    continue;
                };
                let Some(target_component) = component_name_by_cell.get(target_cell) else {
                    continue;
                };
                if *target_component != component_name {
                    dependencies.push((*target_component).to_string());
                }
            }
        }
        dependencies.sort();
        dependencies.dedup();
    }

    iterative_finish_order(&adjacency)
        .into_iter()
        .filter_map(|name| component_by_name.get(name.as_str()).copied())
        .collect()
}

fn reachable_graph(library: &Library, root_cell: &str) -> ReachableGraph {
    let mut edges = BTreeMap::new();
    let mut dangling = Vec::new();
    let mut pending = vec![root_cell.to_string()];
    let mut visited = HashSet::new();

    while let Some(cell_name) = pending.pop() {
        if !visited.insert(cell_name.clone()) {
            continue;
        }
        let Some(cell) = library.get_cell(&cell_name) else {
            continue;
        };
        let mut cell_edges = Vec::new();
        for (element_index, element) in cell.elements().iter().enumerate() {
            let Element::Reference(reference) = element else {
                continue;
            };
            let Some(target_cell) = terminal_cell_name(reference) else {
                continue;
            };
            if library.get_cell(target_cell).is_some() {
                cell_edges.push(GraphEdge {
                    element_index,
                    target_cell: target_cell.to_string(),
                });
                pending.push(target_cell.to_string());
            } else {
                dangling.push(HierarchyBoundsDiagnostic::DanglingReference {
                    source_cell: cell_name.clone(),
                    element_index,
                    target_cell: target_cell.to_string(),
                });
            }
        }
        edges.insert(cell_name, cell_edges);
    }

    dangling.sort_by(|left, right| diagnostic_key(left).cmp(&diagnostic_key(right)));
    ReachableGraph {
        edges,
        dangling_diagnostics: dangling,
    }
}

fn terminal_cell_name(reference: &Reference) -> Option<&str> {
    let mut current = reference;
    loop {
        match current.instance() {
            Instance::Cell(cell_name) => return Some(cell_name),
            Instance::Element(element) => {
                if let Element::Reference(reference) = element.as_ref().as_ref() {
                    current = reference;
                } else {
                    return None;
                }
            }
        }
    }
}

fn diagnostic_key(diagnostic: &HierarchyBoundsDiagnostic) -> (&str, usize, &str) {
    match diagnostic {
        HierarchyBoundsDiagnostic::DanglingReference {
            source_cell,
            element_index,
            target_cell,
        }
        | HierarchyBoundsDiagnostic::Cycle {
            source_cell,
            element_index,
            target_cell,
            ..
        } => (source_cell, *element_index, target_cell),
        HierarchyBoundsDiagnostic::NonFiniteElement {
            source_cell,
            element_index,
        } => (source_cell, *element_index, ""),
    }
}

fn analyze_sccs(graph: &ReachableGraph) -> SccAnalysis {
    let mut adjacency: BTreeMap<String, Vec<String>> = graph
        .edges
        .keys()
        .map(|cell_name| (cell_name.clone(), Vec::new()))
        .collect();
    let mut reverse = adjacency.clone();
    for (source_cell, edges) in &graph.edges {
        if let Some(targets) = adjacency.get_mut(source_cell) {
            targets.extend(edges.iter().map(|edge| edge.target_cell.clone()));
            targets.sort();
            targets.dedup();
        }
        for edge in edges {
            if let Some(sources) = reverse.get_mut(&edge.target_cell) {
                sources.push(source_cell.clone());
            }
        }
    }
    for sources in reverse.values_mut() {
        sources.sort();
        sources.dedup();
    }

    let finish_order = iterative_finish_order(&adjacency);
    let mut assigned = HashSet::new();
    let mut components = Vec::new();
    for start in finish_order.into_iter().rev() {
        if !assigned.insert(start.clone()) {
            continue;
        }
        let mut members = Vec::new();
        let mut pending = vec![start];
        while let Some(cell_name) = pending.pop() {
            members.push(cell_name.clone());
            if let Some(neighbors) = reverse.get(&cell_name) {
                for neighbor in neighbors.iter().rev() {
                    if assigned.insert(neighbor.clone()) {
                        pending.push(neighbor.clone());
                    }
                }
            }
        }
        members.sort();
        components.push(members);
    }

    let component_by_cell: HashMap<String, usize> = components
        .iter()
        .enumerate()
        .flat_map(|(component_index, cells)| {
            cells
                .iter()
                .cloned()
                .map(move |cell_name| (cell_name, component_index))
        })
        .collect();
    let mut cyclic_component_by_cell = HashMap::new();
    let mut cycles = Vec::new();
    for (component_index, cells) in components.into_iter().enumerate() {
        let self_edge = cells.first().is_some_and(|cell_name| {
            graph
                .edges
                .get(cell_name)
                .is_some_and(|edges| edges.iter().any(|edge| edge.target_cell == *cell_name))
        });
        if cells.len() == 1 && !self_edge {
            continue;
        }
        for cell_name in &cells {
            cyclic_component_by_cell.insert(cell_name.clone(), component_index);
        }
        let representative = cells
            .iter()
            .flat_map(|source_cell| {
                graph
                    .edges
                    .get(source_cell)
                    .into_iter()
                    .flatten()
                    .filter(|edge| {
                        component_by_cell.get(&edge.target_cell) == Some(&component_index)
                    })
                    .map(|edge| {
                        (
                            source_cell.clone(),
                            edge.element_index,
                            edge.target_cell.clone(),
                        )
                    })
            })
            .min();
        if let Some((source_cell, element_index, target_cell)) = representative {
            cycles.push(CycleComponent {
                cells,
                source_cell,
                element_index,
                target_cell,
            });
        }
    }
    cycles.sort_by(|left, right| {
        left.cells.cmp(&right.cells).then_with(|| {
            (&left.source_cell, left.element_index, &left.target_cell).cmp(&(
                &right.source_cell,
                right.element_index,
                &right.target_cell,
            ))
        })
    });

    SccAnalysis {
        cyclic_component_by_cell,
        cycles,
    }
}

fn iterative_finish_order(adjacency: &BTreeMap<String, Vec<String>>) -> Vec<String> {
    let mut visited = HashSet::new();
    let mut finished = Vec::new();
    for start in adjacency.keys() {
        if !visited.insert(start.clone()) {
            continue;
        }
        let mut stack = vec![(start.clone(), 0)];
        while let Some((cell_name, next_neighbor)) = stack.last_mut() {
            let neighbors = adjacency.get(cell_name).map_or(&[][..], Vec::as_slice);
            if let Some(neighbor) = neighbors.get(*next_neighbor) {
                *next_neighbor += 1;
                if visited.insert(neighbor.clone()) {
                    stack.push((neighbor.clone(), 0));
                }
            } else if let Some((cell_name, _)) = stack.pop() {
                finished.push(cell_name);
            }
        }
    }
    finished
}

impl SccAnalysis {
    fn skips_edge(&self, source_cell: &str, target_cell: &str) -> bool {
        self.cyclic_component_by_cell.get(source_cell)
            == self.cyclic_component_by_cell.get(target_cell)
            && self.cyclic_component_by_cell.contains_key(source_cell)
    }
}

fn calculate_cell_bounds(
    library: &Library,
    root_cell: &str,
    scc_analysis: &SccAnalysis,
    state: &mut BoundsState,
) -> Result<(), HierarchyBoundsError> {
    let mut frames = vec![TraversalFrame::new(root_cell.to_string())];
    record_cell_scan(state, root_cell);

    while let Some(frame) = frames.last() {
        let cell = library.get_cell(&frame.cell_name).ok_or_else(|| {
            HierarchyBoundsError::UnknownRoot {
                cell_name: frame.cell_name.clone(),
            }
        })?;
        let action = if let Some(element) = cell.elements().get(frame.next_element_index) {
            match element {
                Element::Reference(reference) => {
                    let location = ReferenceLocation {
                        cell_name: frame.cell_name.clone(),
                        element_index: frame.next_element_index,
                    };
                    match evaluate_reference(
                        library,
                        reference,
                        &ReferenceContext {
                            source_cell: &frame.cell_name,
                            scc_analysis,
                        },
                        state,
                    ) {
                        ReferenceEvaluation::Ready(bounds) => {
                            TraversalAction::Reference(location, bounds)
                        }
                        ReferenceEvaluation::NeedCell(cell_name) => {
                            TraversalAction::Descend(cell_name)
                        }
                    }
                }
                _ => TraversalAction::Direct(direct_element_bounds(element)),
            }
        } else {
            TraversalAction::Complete
        };

        match action {
            TraversalAction::Complete => {
                if let Some(frame) = frames.pop() {
                    state.cell_bounds.insert(frame.cell_name, frame.bounds);
                }
            }
            TraversalAction::Direct(Ok(bounds)) => {
                if let Some(frame) = frames.last_mut() {
                    if let Some(bounds) = bounds {
                        merge_bounds(&mut frame.bounds, bounds);
                    }
                    frame.next_element_index += 1;
                }
            }
            TraversalAction::Direct(Err(_)) => {
                if let Some(frame) = frames.last_mut() {
                    state
                        .non_finite_elements
                        .insert((frame.cell_name.clone(), frame.next_element_index));
                    frame.next_element_index += 1;
                }
            }
            TraversalAction::Reference(location, Ok(bounds)) => {
                state.reference_bounds.insert(location, bounds);
                if let Some(frame) = frames.last_mut() {
                    if let Some(bounds) = bounds {
                        merge_bounds(&mut frame.bounds, bounds);
                    }
                    frame.next_element_index += 1;
                }
            }
            TraversalAction::Reference(location, Err(_)) => {
                state.reference_bounds.insert(location.clone(), None);
                state
                    .non_finite_elements
                    .insert((location.cell_name, location.element_index));
                if let Some(frame) = frames.last_mut() {
                    frame.next_element_index += 1;
                }
            }
            TraversalAction::Descend(cell_name) => {
                record_cell_scan(state, &cell_name);
                frames.push(TraversalFrame::new(cell_name));
            }
        }
    }

    Ok(())
}

fn evaluate_reference(
    library: &Library,
    reference: &Reference,
    context: &ReferenceContext<'_>,
    state: &BoundsState,
) -> ReferenceEvaluation {
    match reference.instance() {
        Instance::Cell(target_cell) => {
            match evaluate_cell_source(library, target_cell, context, state) {
                ReferenceEvaluation::Ready(bounds) => {
                    ReferenceEvaluation::Ready(transform_bounds_result(bounds, reference.grid()))
                }
                evaluation @ ReferenceEvaluation::NeedCell(_) => evaluation,
            }
        }
        Instance::Element(element) => match element.as_ref().as_ref() {
            Element::Reference(nested) => {
                evaluate_inline_reference_chain(library, reference.grid(), nested, context, state)
            }
            element => ReferenceEvaluation::Ready(transform_bounds_result(
                direct_element_bounds(element),
                reference.grid(),
            )),
        },
    }
}

fn evaluate_inline_reference_chain(
    library: &Library,
    outer_grid: &Grid,
    reference: &Reference,
    context: &ReferenceContext<'_>,
    state: &BoundsState,
) -> ReferenceEvaluation {
    let mut current = reference;
    let mut grids = vec![outer_grid];
    let source_bounds = loop {
        grids.push(current.grid());
        match current.instance() {
            Instance::Cell(target_cell) => {
                match evaluate_cell_source(library, target_cell, context, state) {
                    ReferenceEvaluation::Ready(bounds) => break bounds,
                    evaluation @ ReferenceEvaluation::NeedCell(_) => return evaluation,
                }
            }
            Instance::Element(element) => match element.as_ref().as_ref() {
                Element::Reference(reference) => current = reference,
                element => break direct_element_bounds(element),
            },
        }
    };

    let mut bounds = source_bounds;
    for grid in grids.into_iter().rev() {
        bounds = transform_bounds_result(bounds, grid);
    }
    ReferenceEvaluation::Ready(bounds)
}

fn evaluate_cell_source(
    library: &Library,
    target_cell: &str,
    context: &ReferenceContext<'_>,
    state: &BoundsState,
) -> ReferenceEvaluation {
    if context
        .scc_analysis
        .skips_edge(context.source_cell, target_cell)
    {
        return ReferenceEvaluation::Ready(Ok(None));
    }
    if let Some(bounds) = state.cell_bounds.get(target_cell) {
        return ReferenceEvaluation::Ready(Ok(*bounds));
    }
    if library.get_cell(target_cell).is_none() {
        return ReferenceEvaluation::Ready(Ok(None));
    }
    ReferenceEvaluation::NeedCell(target_cell.to_string())
}

fn record_cell_scan(state: &mut BoundsState, cell_name: &str) {
    #[cfg(test)]
    state
        .cell_scan_counts
        .entry(cell_name.to_string())
        .and_modify(|count| *count += 1)
        .or_insert(1);
    #[cfg(not(test))]
    let _ = (state, cell_name);
}

fn transform_bounds_result(bounds: BoundsResult, grid: &Grid) -> BoundsResult {
    transform_bounds_by_grid(bounds?, grid)
}

fn transform_bounds_by_grid(bounds: Option<WorldBounds>, grid: &Grid) -> BoundsResult {
    let angle = grid.angle().value();
    let magnification = grid.magnification();
    let origin = grid.origin();
    let origin_x = origin.x().absolute_value();
    let origin_y = origin.y().absolute_value();
    let spacing_x = grid.spacing_x().unwrap_or_default();
    let spacing_y = grid.spacing_y().unwrap_or_default();
    let spacing_x = (
        spacing_x.x().absolute_value(),
        spacing_x.y().absolute_value(),
    );
    let spacing_y = (
        spacing_y.x().absolute_value(),
        spacing_y.y().absolute_value(),
    );
    if ![
        angle,
        magnification,
        origin_x,
        origin_y,
        spacing_x.0,
        spacing_x.1,
        spacing_y.0,
        spacing_y.1,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        return Err(NonFiniteBounds);
    }
    if grid.columns() == 0 || grid.rows() == 0 {
        return Ok(None);
    }
    let Some(bounds) = bounds else {
        return Ok(None);
    };
    if !bounds.is_finite() {
        return Err(NonFiniteBounds);
    }

    let (sin_angle, cos_angle) = angle.sin_cos();
    let mut result = None;
    for column in [0, grid.columns() - 1] {
        for row in [0, grid.rows() - 1] {
            let offset_x = spacing_x.0 * f64::from(column) + spacing_y.0 * f64::from(row);
            let offset_y = spacing_x.1 * f64::from(column) + spacing_y.1 * f64::from(row);
            let rotated_offset_x = offset_x * cos_angle - offset_y * sin_angle;
            let rotated_offset_y = offset_x * sin_angle + offset_y * cos_angle;

            for (mut x, mut y) in bounds.corners() {
                if grid.x_reflection() {
                    y = -y;
                }
                (x, y) = (
                    (x * cos_angle - y * sin_angle) * magnification,
                    (x * sin_angle + y * cos_angle) * magnification,
                );
                let x = x + origin_x + rotated_offset_x;
                let y = y + origin_y + rotated_offset_y;
                if !x.is_finite() || !y.is_finite() {
                    return Err(NonFiniteBounds);
                }
                merge_point(&mut result, x, y);
            }
        }
    }
    Ok(result)
}

fn direct_element_bounds(element: &Element) -> BoundsResult {
    match element {
        Element::Polygon(polygon) => point_bounds(polygon.points().iter()),
        Element::Box(gds_box) => point_bounds(gds_box.points().iter()),
        Element::Node(node) => point_bounds(node.points().iter()),
        Element::Path(path) => path_bounds(path),
        Element::Text(text) => point_bounds(std::iter::once(text.origin())),
        Element::Reference(_) => Ok(None),
    }
}

fn point_bounds<'a>(points: impl IntoIterator<Item = &'a Point>) -> BoundsResult {
    let mut bounds = None;
    for point in points {
        let x = point.x().absolute_value();
        let y = point.y().absolute_value();
        if !x.is_finite() || !y.is_finite() {
            return Err(NonFiniteBounds);
        }
        merge_point(&mut bounds, x, y);
    }
    Ok(bounds)
}

fn path_bounds(path: &crate::Path) -> BoundsResult {
    let mut bounds = point_bounds(path.points().iter())?;
    let half_width = path
        .width()
        .map_or(0.0, |width| width.absolute_value().abs() / 2.0);
    let begin_extension = path
        .begin_extension()
        .map_or(0.0, |extension| extension.absolute_value().max(0.0));
    let end_extension = path
        .end_extension()
        .map_or(0.0, |extension| extension.absolute_value().max(0.0));
    let padding = half_width + begin_extension.max(end_extension);
    if !padding.is_finite() {
        return Err(NonFiniteBounds);
    }
    if let Some(bounds) = &mut bounds {
        bounds.min_x -= padding;
        bounds.min_y -= padding;
        bounds.max_x += padding;
        bounds.max_y += padding;
        if !bounds.is_finite() {
            return Err(NonFiniteBounds);
        }
    }
    Ok(bounds)
}

fn merge_bounds(result: &mut Option<WorldBounds>, bounds: WorldBounds) {
    *result = Some(match result {
        Some(current) => current.merge(&bounds),
        None => bounds,
    });
}

fn merge_point(result: &mut Option<WorldBounds>, x: f64, y: f64) {
    merge_bounds(result, WorldBounds::new(x, y, x, y));
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use crate::{
        Cell, DataType, GdsBox, HorizontalPresentation, Layer, Node, Path, Point, Polygon, Radians,
        Text, Unit, VerticalPresentation,
    };

    use super::*;

    fn point(x: f64, y: f64) -> Point {
        Point::float(x, y, 1.0)
    }

    fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Polygon {
        Polygon::new(
            [
                point(min_x, min_y),
                point(max_x, min_y),
                point(max_x, max_y),
                point(min_x, max_y),
            ],
            Layer::new(1),
            DataType::new(0),
        )
    }

    fn assert_bounds(actual: Option<WorldBounds>, expected: (f64, f64, f64, f64)) {
        let actual = actual.expect("bounds should exist");
        let tolerance = 1e-12;
        assert!((actual.min_x - expected.0).abs() < tolerance, "{actual:?}");
        assert!((actual.min_y - expected.1).abs() < tolerance, "{actual:?}");
        assert!((actual.max_x - expected.2).abs() < tolerance, "{actual:?}");
        assert!((actual.max_y - expected.3).abs() < tolerance, "{actual:?}");
    }

    fn assert_element_bounds(element: impl Into<Element>, expected: (f64, f64, f64, f64)) {
        let mut cell = Cell::new("element");
        cell.add(element);
        let mut library = Library::new("bounds");
        library.add_cell(cell);
        let report = library
            .hierarchy_bounds("element")
            .expect("direct element should resolve");
        assert_bounds(report.root_bounds(), expected);
    }

    #[test]
    fn direct_and_empty_cells_have_explicit_bounds() {
        let mut direct = Cell::new("direct");
        direct.add(rectangle(-2.0, 3.0, 5.0, 8.0));
        let mut library = Library::new("bounds");
        library.add_cell(Cell::new("empty"));
        library.add_cell(direct);

        let direct_report = library
            .hierarchy_bounds("direct")
            .expect("direct cell should resolve");
        assert_bounds(direct_report.root_bounds(), (-2.0, 3.0, 5.0, 8.0));

        let empty_report = library
            .hierarchy_bounds("empty")
            .expect("empty cell should resolve");
        assert_eq!(empty_report.root_bounds(), None);
        assert_eq!(empty_report.cell_bounds().get("empty"), Some(&None));
    }

    #[test]
    fn direct_path_bounds_include_width_and_extensions() {
        let path = Path::new(
            [point(10.0, 20.0), point(30.0, 40.0)],
            Layer::new(1),
            DataType::new(0),
            None,
            Some(Unit::float(4.0, 1.0)),
            Some(Unit::float(3.0, 1.0)),
            Some(Unit::float(1.0, 1.0)),
        );
        let mut cell = Cell::new("path");
        cell.add(path);
        let mut library = Library::new("bounds");
        library.add_cell(cell);

        let report = library
            .hierarchy_bounds("path")
            .expect("path cell should resolve");
        assert_bounds(report.root_bounds(), (5.0, 15.0, 35.0, 45.0));
    }

    #[test]
    fn direct_elements_compare_extrema_in_physical_units() {
        let low = Point::new(Unit::float(-2.0, 1.0), Unit::float(-7_000.0, 0.001));
        let high = Point::new(Unit::float(5_000.0, 0.001), Unit::float(3.0, 1.0));

        assert_element_bounds(
            Polygon::new(
                [low, high, point(0.0, 0.0)],
                Layer::new(1),
                DataType::new(0),
            ),
            (-2.0, -7.0, 5.0, 3.0),
        );
        assert_element_bounds(
            GdsBox::new(low, high, Layer::new(1), DataType::new(0)),
            (-2.0, -7.0, 5.0, 3.0),
        );
        assert_element_bounds(
            Node::new(vec![low, high], Layer::new(1), DataType::new(0)),
            (-2.0, -7.0, 5.0, 3.0),
        );
        assert_element_bounds(
            Path::new(
                [low, high],
                Layer::new(1),
                DataType::new(0),
                None,
                None,
                None,
                None,
            ),
            (-2.0, -7.0, 5.0, 3.0),
        );
        assert_element_bounds(
            Text::new(
                "mixed",
                low,
                Layer::new(1),
                DataType::new(0),
                1.0,
                Radians::new(0.0),
                false,
                VerticalPresentation::Middle,
                HorizontalPresentation::Centre,
            ),
            (-2.0, -7.0, -2.0, -7.0),
        );
    }

    #[test]
    fn negative_path_values_never_shrink_bounds() {
        let path = Path::new(
            [point(0.0, 0.0), point(10.0, 0.0)],
            Layer::new(1),
            DataType::new(0),
            None,
            Some(Unit::float(-4.0, 0.5)),
            Some(Unit::float(-100.0, 1.0)),
            Some(Unit::float(2.0, 0.5)),
        );

        assert_element_bounds(path, (-2.0, -2.0, 12.0, 2.0));
    }

    #[test]
    fn nested_reference_applies_all_transform_components() {
        let mut leaf = Cell::new("leaf");
        leaf.add(rectangle(0.0, 0.0, 2.0, 1.0));
        let mut middle = Cell::new("middle");
        middle.add(
            Reference::new("leaf").with_grid(
                Grid::default()
                    .with_origin(point(10.0, 20.0))
                    .with_angle(Radians::new(FRAC_PI_2))
                    .with_magnification(2.0)
                    .with_x_reflection(true),
            ),
        );
        let mut top = Cell::new("top");
        top.add(Reference::new("middle").with_grid(Grid::default().with_origin(point(-5.0, 3.0))));
        let mut library = Library::new("bounds");
        library.add_cell(leaf);
        library.add_cell(middle);
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("hierarchy should resolve");
        assert_bounds(report.root_bounds(), (5.0, 23.0, 7.0, 27.0));
    }

    #[test]
    fn inline_reference_chain_resolves_named_cell_without_extra_entries() {
        let mut leaf = Cell::new("leaf");
        leaf.add(rectangle(0.0, 0.0, 1.0, 1.0));
        let nested = Reference::new("leaf").with_grid(Grid::default().with_origin(point(2.0, 0.0)));
        let mut top = Cell::new("top");
        top.add(Reference::new(nested).with_grid(Grid::default().with_origin(point(3.0, 0.0))));
        let mut library = Library::new("bounds");
        library.add_cell(leaf);
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("inline hierarchy should resolve");
        assert_bounds(report.root_bounds(), (5.0, 0.0, 6.0, 1.0));
        assert_eq!(report.reference_bounds().len(), 1);
    }

    #[test]
    fn rotated_bounds_use_all_four_source_corners() {
        let bounds = WorldBounds::new(0.0, 0.0, 2.0, 1.0);
        let transformed = bounds.transformed_by_grid(
            &Grid::default().with_angle(Radians::new(std::f64::consts::FRAC_PI_4)),
        );
        let sqrt_two = 2.0_f64.sqrt();
        assert_bounds(
            transformed,
            (-sqrt_two / 2.0, 0.0, sqrt_two, 3.0 * sqrt_two / 2.0),
        );
    }

    #[test]
    fn skew_and_negative_array_spacing_use_extreme_members() {
        let mut leaf = Cell::new("leaf");
        leaf.add(rectangle(0.0, 0.0, 2.0, 1.0));
        let mut top = Cell::new("top");
        top.add(
            Reference::new("leaf").with_grid(
                Grid::default()
                    .with_columns(3)
                    .with_rows(2)
                    .with_spacing_x(Some(point(-4.0, 1.0)))
                    .with_spacing_y(Some(point(2.0, -5.0))),
            ),
        );
        let mut library = Library::new("bounds");
        library.add_cell(leaf);
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("hierarchy should resolve");
        assert_bounds(report.root_bounds(), (-8.0, -5.0, 4.0, 3.0));
        assert_eq!(report.reference_bounds().len(), 1);
        assert_bounds(
            report.reference_bounds().values().next().copied().flatten(),
            (-8.0, -5.0, 4.0, 3.0),
        );
    }

    #[test]
    fn large_aref_stores_one_aggregate_reference_bound() {
        let mut leaf = Cell::new("leaf");
        leaf.add(rectangle(0.0, 0.0, 1.0, 1.0));
        let mut top = Cell::new("top");
        top.add(
            Reference::new("leaf").with_grid(
                Grid::default()
                    .with_columns(1_000_000)
                    .with_rows(1_000_000)
                    .with_spacing_x(Some(point(2.0, 0.0)))
                    .with_spacing_y(Some(point(0.0, -3.0))),
            ),
        );
        let mut library = Library::new("bounds");
        library.add_cell(leaf);
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("hierarchy should resolve");
        assert_eq!(report.reference_bounds().len(), 1);
        assert_bounds(report.root_bounds(), (0.0, -2_999_997.0, 1_999_999.0, 1.0));
    }

    #[test]
    fn repeated_child_is_reported_once_and_each_reference_has_bounds() {
        let mut leaf = Cell::new("leaf");
        leaf.add(rectangle(0.0, 0.0, 1.0, 1.0));
        let mut top = Cell::new("top");
        for index in 0..100 {
            top.add(
                Reference::new("leaf")
                    .with_grid(Grid::default().with_origin(point(f64::from(index), 0.0))),
            );
        }
        let mut library = Library::new("bounds");
        library.add_cell(leaf);
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("hierarchy should resolve");
        assert_eq!(report.cell_bounds().len(), 2);
        assert_eq!(report.reference_bounds().len(), 100);
        assert_bounds(report.root_bounds(), (0.0, 0.0, 100.0, 1.0));

        let (state, _) = calculate_bounds(&library, "top").expect("hierarchy should resolve");
        assert_eq!(state.cell_scan_counts.get("leaf"), Some(&1));
        assert_eq!(state.cell_scan_counts.get("top"), Some(&1));
    }

    #[test]
    fn deep_hierarchy_uses_heap_backed_traversal() {
        const DEPTH: usize = 10_000;

        let mut library = Library::new("bounds");
        for index in 0..DEPTH {
            let cell_name = format!("cell_{index:05}");
            let mut cell = Cell::new(&cell_name);
            if index + 1 == DEPTH {
                cell.add(rectangle(0.0, 0.0, 1.0, 1.0));
                cell.add(Reference::new("missing"));
            } else {
                cell.add(Reference::new(format!("cell_{:05}", index + 1)));
            }
            library.add_cell(cell);
        }

        let report = library
            .hierarchy_bounds("cell_00000")
            .expect("deep hierarchy should resolve");
        assert_bounds(report.root_bounds(), (0.0, 0.0, 1.0, 1.0));
        assert_eq!(report.cell_bounds().len(), DEPTH);
        assert_eq!(
            report.diagnostics(),
            &[HierarchyBoundsDiagnostic::DanglingReference {
                source_cell: "cell_09999".to_string(),
                element_index: 1,
                target_cell: "missing".to_string(),
            }],
        );
    }

    #[test]
    fn dangling_reference_keeps_valid_sibling_geometry() {
        let mut top = Cell::new("top");
        top.add(rectangle(1.0, 2.0, 3.0, 4.0));
        top.add(Reference::new("missing"));
        let mut library = Library::new("bounds");
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("known root should produce a partial report");
        assert_bounds(report.root_bounds(), (1.0, 2.0, 3.0, 4.0));
        insta::assert_debug_snapshot!(report.diagnostics(), @r#"
        [
            DanglingReference {
                source_cell: "top",
                element_index: 1,
                target_cell: "missing",
            },
        ]
        "#);
    }

    #[test]
    fn diagnostic_order_is_canonical_across_cell_storage_order() {
        let mut child_a = Cell::new("a");
        child_a.add(Reference::new("missing_a"));
        let mut child_b = Cell::new("b");
        child_b.add(Reference::new("missing_b"));
        let mut top = Cell::new("top");
        top.add(Reference::new("b"));
        top.add(Reference::new("a"));
        let mut library = Library::new("bounds");
        library.add_cell(child_a);
        library.add_cell(top);
        library.add_cell(child_b);

        let report = library
            .hierarchy_bounds("top")
            .expect("known root should produce a partial report");
        let targets: Vec<&str> = report
            .diagnostics()
            .iter()
            .filter_map(|diagnostic| {
                if let HierarchyBoundsDiagnostic::DanglingReference { target_cell, .. } = diagnostic
                {
                    Some(target_cell.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(targets, ["missing_a", "missing_b"]);
    }

    #[test]
    fn self_cycle_keeps_valid_geometry_and_reports_component() {
        let mut cell = Cell::new("self");
        cell.add(rectangle(0.0, 0.0, 2.0, 2.0));
        cell.add(Reference::new("self"));
        let mut library = Library::new("bounds");
        library.add_cell(cell);

        let report = library
            .hierarchy_bounds("self")
            .expect("known root should produce a partial report");
        assert_bounds(report.root_bounds(), (0.0, 0.0, 2.0, 2.0));
        insta::assert_debug_snapshot!(report.diagnostics(), @r#"
        [
            Cycle {
                cells: [
                    "self",
                ],
                source_cell: "self",
                element_index: 1,
                target_cell: "self",
            },
        ]
        "#);
    }

    #[test]
    fn mutual_cycle_is_deterministic() {
        let mut a = Cell::new("a");
        a.add(Reference::new("b"));
        let mut b = Cell::new("b");
        b.add(Reference::new("a"));
        let mut library = Library::new("bounds");
        library.add_cell(b);
        library.add_cell(a);

        let report = library
            .hierarchy_bounds("a")
            .expect("known root should produce a partial report");
        assert_eq!(report.root_bounds(), None);
        insta::assert_debug_snapshot!(report.diagnostics(), @r#"
        [
            Cycle {
                cells: [
                    "a",
                    "b",
                ],
                source_cell: "a",
                element_index: 0,
                target_cell: "b",
            },
        ]
        "#);
    }

    #[test]
    fn cyclic_component_bounds_are_root_and_sibling_order_independent() {
        fn library(reverse_top_references: bool) -> Library {
            let mut a = Cell::new("a");
            a.add(rectangle(0.0, 0.0, 1.0, 1.0));
            a.add(Reference::new(Reference::new("c")));
            let mut c = Cell::new("c");
            c.add(rectangle(10.0, 20.0, 11.0, 21.0));
            c.add(Reference::new("a"));
            let mut top = Cell::new("top");
            let references = if reverse_top_references {
                ["c", "a"]
            } else {
                ["a", "c"]
            };
            for target in references {
                top.add(Reference::new(target));
            }
            let mut library = Library::new("bounds");
            library.add_cell(c);
            library.add_cell(top);
            library.add_cell(a);
            library
        }

        let ordered_library = library(false);
        let from_a = ordered_library
            .hierarchy_bounds("a")
            .expect("cyclic component should produce partial bounds");
        let from_c = ordered_library
            .hierarchy_bounds("c")
            .expect("cyclic component should produce partial bounds");
        let from_top = ordered_library
            .hierarchy_bounds("top")
            .expect("parent should produce partial bounds");
        let reversed_top = library(true)
            .hierarchy_bounds("top")
            .expect("reordered parent should produce partial bounds");

        for report in [&from_a, &from_c, &from_top, &reversed_top] {
            assert_bounds(
                report.cell_bounds().get("a").copied().flatten(),
                (0.0, 0.0, 11.0, 21.0),
            );
            assert_bounds(
                report.cell_bounds().get("c").copied().flatten(),
                (0.0, 0.0, 11.0, 21.0),
            );
            assert_eq!(
                report.reference_bounds().get(&ReferenceLocation {
                    cell_name: "a".to_string(),
                    element_index: 1,
                }),
                Some(&Some(WorldBounds::new(0.0, 0.0, 11.0, 21.0))),
            );
            assert_eq!(
                report.reference_bounds().get(&ReferenceLocation {
                    cell_name: "c".to_string(),
                    element_index: 1,
                }),
                Some(&Some(WorldBounds::new(0.0, 0.0, 11.0, 21.0))),
            );
            assert_eq!(report.diagnostics(), from_a.diagnostics());
        }
    }

    #[test]
    fn parent_bounds_cover_geometry_reached_before_cycle_cutoff() {
        let mut a = Cell::new("a");
        a.add(rectangle(0.0, 0.0, 1.0, 1.0));
        a.add(Reference::new("b"));
        let mut b = Cell::new("b");
        b.add(rectangle(100.0, 0.0, 101.0, 1.0));
        b.add(Reference::new("a"));
        let mut top = Cell::new("top");
        top.add(Reference::new("a"));
        let mut library = Library::new("bounds");
        library.add_cell(top);
        library.add_cell(b);
        library.add_cell(a);

        let report = library
            .hierarchy_bounds("top")
            .expect("known root should produce conservative cyclic bounds");

        for cell_name in ["top", "a", "b"] {
            assert_bounds(
                report.cell_bounds().get(cell_name).copied().flatten(),
                (0.0, 0.0, 101.0, 1.0),
            );
        }
        for cell_name in ["a", "b"] {
            assert_eq!(
                report.reference_bounds().get(&ReferenceLocation {
                    cell_name: cell_name.to_string(),
                    element_index: 1,
                }),
                Some(&Some(WorldBounds::new(0.0, 0.0, 101.0, 1.0))),
            );
        }
        insta::assert_debug_snapshot!(report.diagnostics(), @r#"
        [
            Cycle {
                cells: [
                    "a",
                    "b",
                ],
                source_cell: "a",
                element_index: 1,
                target_cell: "b",
            },
        ]
        "#);
    }

    #[test]
    fn dependent_cyclic_components_expand_bottom_up() {
        let mut a1 = Cell::new("a1");
        a1.add(Reference::new("a2"));
        a1.add(Reference::new("b1"));
        let mut a2 = Cell::new("a2");
        a2.add(Reference::new("a1"));
        let mut b1 = Cell::new("b1");
        b1.add(Reference::new("b2"));
        let mut b2 = Cell::new("b2");
        b2.add(rectangle(100.0, 0.0, 101.0, 1.0));
        b2.add(Reference::new("b1"));
        let mut top = Cell::new("top");
        top.add(Reference::new("a1"));
        let mut library = Library::new("bounds");
        for cell in [top, b2, b1, a2, a1] {
            library.add_cell(cell);
        }

        let report = library
            .hierarchy_bounds("top")
            .expect("known root should produce conservative cyclic bounds");

        assert_bounds(report.root_bounds(), (100.0, 0.0, 101.0, 1.0));
    }

    #[test]
    fn non_finite_elements_are_reported_and_never_returned() {
        let mut top = Cell::new("top");
        top.add(Polygon::new(
            [
                Point::float(f64::NAN, 0.0, 1.0),
                point(1.0, 1.0),
                point(2.0, 0.0),
            ],
            Layer::new(1),
            DataType::new(0),
        ));
        top.add(rectangle(10.0, 20.0, 30.0, 40.0));
        top.add(
            Reference::new(rectangle(0.0, 0.0, 1.0, 1.0))
                .with_grid(Grid::default().with_magnification(f64::INFINITY)),
        );
        let mut library = Library::new("bounds");
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("invalid elements should be non-fatal");
        assert_bounds(report.root_bounds(), (10.0, 20.0, 30.0, 40.0));
        insta::assert_debug_snapshot!(report.diagnostics(), @r#"
        [
            NonFiniteElement {
                source_cell: "top",
                element_index: 0,
            },
            NonFiniteElement {
                source_cell: "top",
                element_index: 2,
            },
        ]
        "#);
        assert!(
            WorldBounds::new(f64::NAN, 0.0, 1.0, 1.0)
                .transformed_by_grid(&Grid::default())
                .is_none()
        );
    }

    #[test]
    fn ten_thousand_nested_inline_references_use_heap_traversal() {
        const DEPTH: usize = 10_000;

        let mut leaf = Cell::new("leaf");
        leaf.add(rectangle(0.0, 0.0, 1.0, 1.0));
        let mut reference = Reference::new("leaf");
        for _ in 1..DEPTH {
            reference = Reference::new(reference);
        }
        let mut top = Cell::new("top");
        top.add(reference);
        let mut library = Library::new("bounds");
        library.add_cell(leaf);
        library.add_cell(top);

        let report = library
            .hierarchy_bounds("top")
            .expect("deep inline hierarchy should resolve");
        assert_bounds(report.root_bounds(), (0.0, 0.0, 1.0, 1.0));
        assert_eq!(report.reference_bounds().len(), 1);
    }

    #[test]
    fn unknown_root_is_structured() {
        let library = Library::new("bounds");
        insta::assert_debug_snapshot!(library.hierarchy_bounds("missing"), @r#"
        Err(
            UnknownRoot {
                cell_name: "missing",
            },
        )
        "#);
    }

    #[test]
    fn later_call_recomputes_after_mutation() {
        let mut cell = Cell::new("top");
        cell.add(rectangle(0.0, 0.0, 1.0, 1.0));
        let mut library = Library::new("bounds");
        library.add_cell(cell);

        let before = library
            .hierarchy_bounds("top")
            .expect("first query should resolve");
        assert_bounds(before.root_bounds(), (0.0, 0.0, 1.0, 1.0));

        library
            .get_cell_mut("top")
            .expect("cell should exist")
            .add(rectangle(10.0, 20.0, 30.0, 40.0));
        let after = library
            .hierarchy_bounds("top")
            .expect("second query should resolve");
        assert_bounds(after.root_bounds(), (0.0, 0.0, 30.0, 40.0));
    }
}
