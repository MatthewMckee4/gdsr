use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;

use gdsr::{Cell, CellStats, DataType, Element, Layer, Library, Movable, Point};

use crate::colors::LayerColorMap;
use crate::drawable::Drawable;
use crate::hierarchy::{self, CellTreeNode, ExpandState};
use crate::spatial::SpatialGrid;
use crate::viewport;

/// Tracks an in-flight file-open operation.
#[derive(Default)]
pub struct FileLoadState {
    pub file_path: Option<PathBuf>,
    pub load_receiver: Option<(PathBuf, mpsc::Receiver<Result<Library, String>>)>,
    pub loading: bool,
    pub error_message: Option<String>,
}

/// Holds the loaded library, selected cell, and render indexes.
pub struct CellState {
    pub library: Library,
    pub cell_names: Vec<String>,
    pub cell_tree: Vec<CellTreeNode>,
    pub flat_tree: Vec<CellTreeNode>,
    pub expand_state: ExpandState,
    pub selected_cell: Option<String>,
    pub elements: Vec<Element>,
    pub layers: BTreeSet<(Layer, DataType)>,
    pub spatial_grid: Option<SpatialGrid>,
    pub tessellation_cache: HashMap<u32, Vec<usize>>,
    pub cell_stats: Option<CellStats>,
    pub render_depth: u32,
}

impl CellState {
    pub fn new(library: Library) -> Self {
        let expand_state = ExpandState::default();
        let mut state = Self {
            library,
            cell_names: Vec::new(),
            cell_tree: Vec::new(),
            flat_tree: Vec::new(),
            expand_state,
            selected_cell: None,
            elements: Vec::new(),
            layers: BTreeSet::new(),
            spatial_grid: None,
            tessellation_cache: HashMap::new(),
            cell_stats: None,
            render_depth: 1,
        };
        state.rebuild_cell_indexes();
        state
    }

    fn rebuild_cell_indexes(&mut self) {
        self.cell_names = self.library.cells().keys().cloned().collect();
        self.cell_names.sort();
        self.cell_tree = hierarchy::build_cell_tree(&self.library);
        self.flat_tree = hierarchy::build_flat_cell_tree(&self.library);
    }

    pub fn delete_element(&mut self, index: usize) -> bool {
        let Some(cell_name) = self.selected_cell.clone() else {
            return false;
        };

        let removed = self
            .library
            .get_cell_mut(&cell_name)
            .and_then(|cell| cell.remove_element(index))
            .is_some();

        removed && self.load_direct_cell_elements(&cell_name)
    }

    pub fn load_direct_cell_elements(&mut self, name: &str) -> bool {
        self.cell_stats = self.library.get_cell(name).map(CellStats::from_cell);

        let Some(cell) = self.library.get_cell(name) else {
            self.elements.clear();
            self.rebuild_render_indexes();
            return false;
        };
        self.elements = cell.elements().to_vec();
        self.rebuild_render_indexes();

        self.refresh_layers_for(name);
        true
    }

    pub fn refresh_layers(&mut self) {
        let Some(name) = self.selected_cell.clone() else {
            self.layers.clear();
            return;
        };
        self.refresh_layers_for(&name);
    }

    fn refresh_layers_for(&mut self, name: &str) {
        self.layers.clear();
        let mut visiting = HashSet::new();
        let mut visited_depths = HashMap::new();
        collect_layers_from_cell(
            name,
            &self.library,
            self.render_depth,
            &mut self.layers,
            &mut visiting,
            &mut visited_depths,
        );
    }

    pub fn insert_element(&mut self, index: usize, element: Element) -> bool {
        let Some(cell_name) = self.selected_cell.clone() else {
            return false;
        };

        let inserted = self
            .library
            .get_cell_mut(&cell_name)
            .is_some_and(|cell| cell.insert_element(index, element));

        inserted && self.load_direct_cell_elements(&cell_name)
    }

    pub fn create_cell(&mut self, name: &str) -> bool {
        if name.is_empty() || self.library.cells().contains_key(name) {
            return false;
        }

        self.library.add_cell(Cell::new(name));
        self.rebuild_cell_indexes();
        self.selected_cell = Some(name.to_string());
        self.expand_state.set_expanded(name, true);
        self.load_direct_cell_elements(name)
    }

    pub fn rename_cell(&mut self, old_name: &str, new_name: &str) -> bool {
        if new_name.is_empty() || !self.library.rename_cell(old_name, new_name) {
            return false;
        }

        self.rebuild_cell_indexes();
        if self.selected_cell.as_deref() == Some(old_name) {
            self.selected_cell = Some(new_name.to_string());
        }
        self.expand_state.set_expanded(old_name, false);
        self.expand_state.set_expanded(new_name, true);

        if let Some(name) = self.selected_cell.clone() {
            self.load_direct_cell_elements(&name)
        } else {
            true
        }
    }

    pub fn move_element(&mut self, index: usize, delta: Point) -> bool {
        let Some(cell_name) = self.selected_cell.clone() else {
            return false;
        };

        let moved = if let Some(cell) = self.library.get_cell_mut(&cell_name)
            && let Some(element) = cell.element_mut(index)
        {
            *element = element.clone().move_by(delta);
            true
        } else {
            false
        };

        moved && self.load_direct_cell_elements(&cell_name)
    }

    fn rebuild_render_indexes(&mut self) {
        self.layers.clear();
        for element in &self.elements {
            self.layers.extend(element.layer_keys());
        }
        self.spatial_grid = viewport::compute_bounds(&self.elements)
            .map(|bounds| SpatialGrid::build(&self.elements, &bounds));
        self.tessellation_cache.clear();
    }
}

fn collect_layers_from_cell(
    cell_name: &str,
    library: &Library,
    depth: u32,
    layers: &mut BTreeSet<(Layer, DataType)>,
    visiting: &mut HashSet<String>,
    visited_depths: &mut HashMap<String, u32>,
) {
    if depth == 0
        || visited_depths
            .get(cell_name)
            .is_some_and(|visited_depth| *visited_depth >= depth)
        || !visiting.insert(cell_name.to_owned())
    {
        return;
    }
    visited_depths.insert(cell_name.to_owned(), depth);

    if let Some(cell) = library.get_cell(cell_name) {
        for element in cell.iter_elements() {
            collect_layers_from_element(element, library, depth, layers, visiting, visited_depths);
        }
    }

    visiting.remove(cell_name);
}

fn collect_layers_from_element(
    element: &Element,
    library: &Library,
    depth: u32,
    layers: &mut BTreeSet<(Layer, DataType)>,
    visiting: &mut HashSet<String>,
    visited_depths: &mut HashMap<String, u32>,
) {
    let Element::Reference(reference) = element else {
        layers.extend(element.layer_keys());
        return;
    };

    if depth <= 1 {
        return;
    }

    if let Some(cell_name) = reference.instance().as_cell() {
        collect_layers_from_cell(
            cell_name,
            library,
            depth - 1,
            layers,
            visiting,
            visited_depths,
        );
    } else if let Some(inner) = reference.instance().as_element() {
        collect_layers_from_element(
            inner.as_ref().as_ref(),
            library,
            depth - 1,
            layers,
            visiting,
            visited_depths,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdsr::{Cell, Element, Library, Reference};

    use crate::testutil::helpers::polygon;

    fn cell_state_with_elements(elements: Vec<Element>) -> CellState {
        let mut source_cell = Cell::new("top");
        for element in elements {
            source_cell.add(element);
        }

        let mut library = Library::new("test");
        library.add_cell(source_cell);

        let mut cell = CellState::new(library);
        cell.selected_cell = Some("top".to_string());
        assert!(cell.load_direct_cell_elements("top"));
        cell
    }

    #[test]
    fn delete_element_rebuilds_render_indexes() {
        let mut cell = cell_state_with_elements(vec![
            polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0),
            polygon(vec![(1000, 1000), (1100, 1000), (1100, 1100)], 2, 0),
        ]);
        cell.tessellation_cache.insert(0, vec![0, 1, 2]);
        cell.rebuild_render_indexes();

        assert!(cell.delete_element(0));

        assert_eq!(cell.elements.len(), 1);
        assert_eq!(
            cell.library
                .get_cell("top")
                .expect("top cell should exist")
                .elements()
                .len(),
            1
        );
        assert_eq!(
            cell.layers,
            BTreeSet::from([(Layer::new(2), DataType::new(0))])
        );
        assert!(cell.spatial_grid.is_some());
        assert!(cell.tessellation_cache.is_empty());
    }

    #[test]
    fn create_cell_updates_cell_indexes_and_selects_new_cell() {
        let mut cell = CellState::new(Library::new("test"));

        assert!(cell.create_cell("new_cell"));

        assert!(cell.library.get_cell("new_cell").is_some());
        assert_eq!(cell.cell_names, vec!["new_cell"]);
        assert_eq!(cell.selected_cell.as_deref(), Some("new_cell"));
        assert!(cell.elements.is_empty());
        assert!(cell.spatial_grid.is_none());
    }

    #[test]
    fn refreshing_depth_layers_preserves_direct_render_indexes() {
        let mut leaf = Cell::new("leaf");
        leaf.add(polygon(vec![(0, 0), (100, 0), (100, 100)], 2, 0));
        let mut top = Cell::new("top");
        top.add(polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0));
        top.add(Reference::new("leaf"));
        let mut library = Library::new("test");
        library.add_cell(leaf);
        library.add_cell(top);
        let mut cell = CellState::new(library);
        cell.selected_cell = Some("top".to_string());
        assert!(cell.load_direct_cell_elements("top"));
        cell.tessellation_cache.insert(0, vec![0, 1, 2]);

        cell.render_depth = 2;
        cell.refresh_layers();

        assert_eq!(
            cell.layers,
            BTreeSet::from([
                (Layer::new(1), DataType::new(0)),
                (Layer::new(2), DataType::new(0)),
            ])
        );
        assert_eq!(cell.elements.len(), 2);
        assert!(cell.spatial_grid.is_some());
        assert_eq!(
            cell.tessellation_cache.get(&0).map(Vec::as_slice),
            Some(&[0, 1, 2][..])
        );
    }

    #[test]
    fn rename_cell_updates_indexes_selection_and_references() {
        let mut leaf = Cell::new("leaf");
        leaf.add(polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0));
        let mut top = Cell::new("top");
        top.add(Reference::new("leaf"));

        let mut library = Library::new("test");
        library.add_cell(leaf);
        library.add_cell(top);

        let mut cell = CellState::new(library);
        cell.selected_cell = Some("leaf".to_string());
        assert!(cell.load_direct_cell_elements("leaf"));

        assert!(cell.rename_cell("leaf", "renamed"));

        assert!(cell.library.get_cell("leaf").is_none());
        assert!(cell.library.get_cell("renamed").is_some());
        assert_eq!(cell.selected_cell.as_deref(), Some("renamed"));
        assert_eq!(cell.cell_names, vec!["renamed", "top"]);
        assert_eq!(
            cell.library
                .get_cell("top")
                .expect("top cell should exist")
                .referenced_cell_names(),
            vec!["renamed"]
        );
    }

    #[test]
    fn delete_element_rejects_missing_index() {
        let mut cell =
            cell_state_with_elements(vec![polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0)]);
        cell.rebuild_render_indexes();

        assert!(!cell.delete_element(1));

        assert_eq!(cell.elements.len(), 1);
        assert_eq!(
            cell.layers,
            BTreeSet::from([(Layer::new(1), DataType::new(0))])
        );
        assert!(cell.spatial_grid.is_some());
    }

    #[test]
    fn insert_element_rebuilds_render_indexes() {
        let mut cell =
            cell_state_with_elements(vec![polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0)]);
        cell.tessellation_cache.insert(0, vec![0, 1, 2]);

        assert!(cell.insert_element(
            0,
            polygon(vec![(1000, 1000), (1100, 1000), (1100, 1100)], 2, 0)
        ));

        assert_eq!(cell.elements.len(), 2);
        assert_eq!(
            cell.library
                .get_cell("top")
                .expect("top cell should exist")
                .elements()
                .len(),
            2
        );
        assert_eq!(
            cell.layers,
            BTreeSet::from([
                (Layer::new(1), DataType::new(0)),
                (Layer::new(2), DataType::new(0))
            ])
        );
        assert!(cell.spatial_grid.is_some());
        assert!(cell.tessellation_cache.is_empty());
    }

    #[test]
    fn move_element_rebuilds_render_indexes() {
        let mut cell =
            cell_state_with_elements(vec![polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0)]);
        cell.tessellation_cache.insert(0, vec![0, 1, 2]);
        cell.rebuild_render_indexes();

        assert!(cell.move_element(0, Point::default_integer(1000, 2000)));

        let bbox = cell.elements[0]
            .world_bbox()
            .expect("moved element has bbox");
        assert!((bbox.min_x - 1000.0e-9).abs() < 1e-15);
        assert!((bbox.min_y - 2000.0e-9).abs() < 1e-15);
        assert_eq!(
            cell.layers,
            BTreeSet::from([(Layer::new(1), DataType::new(0))])
        );
        assert!(cell.spatial_grid.is_some());
        assert!(cell.tessellation_cache.is_empty());

        let library_bbox = cell
            .library
            .get_cell("top")
            .expect("top cell should exist")
            .elements()[0]
            .world_bbox()
            .expect("moved element has bbox");
        assert!((library_bbox.min_x - 1000.0e-9).abs() < 1e-15);
    }

    #[test]
    fn move_element_rejects_missing_index() {
        let mut cell =
            cell_state_with_elements(vec![polygon(vec![(0, 0), (100, 0), (100, 100)], 1, 0)]);
        cell.rebuild_render_indexes();

        assert!(!cell.move_element(1, Point::default_integer(1000, 2000)));

        let bbox = cell.elements[0]
            .world_bbox()
            .expect("unchanged element has bbox");
        assert!((bbox.min_x - 0.0).abs() < 1e-15);
        assert!(cell.spatial_grid.is_some());
    }

    #[test]
    fn tessellation_cache_returns_same_indices() {
        let coords: Vec<f64> = vec![0.0, 0.0, 100.0, 0.0, 100.0, 100.0, 0.0, 100.0];
        let mut cache: HashMap<u32, Vec<usize>> = HashMap::new();

        let first = cache
            .entry(0)
            .or_insert_with(|| earcutr::earcut(&coords, &[], 2).unwrap_or_default())
            .clone();

        let second = cache
            .entry(0)
            .or_insert_with(|| panic!("should not recompute"))
            .clone();

        assert_eq!(first, second);
        assert!(!first.is_empty());
    }

    #[test]
    fn display_unit_auto_nanometers() {
        insta::assert_snapshot!(
            DisplayUnit::Auto.format_pair(5e-8, -3e-8),
            @"(50.00, -30.00) nm"
        );
    }

    #[test]
    fn display_unit_auto_micrometers() {
        insta::assert_snapshot!(
            DisplayUnit::Auto.format_pair(1.5e-6, 2.0e-5),
            @"(1.500, 20.000) µm"
        );
    }

    #[test]
    fn display_unit_auto_millimeters() {
        insta::assert_snapshot!(
            DisplayUnit::Auto.format_pair(2.5e-3, 1.0e-3),
            @"(2.5000, 1.0000) mm"
        );
    }

    #[test]
    fn display_unit_fixed_nanometers() {
        insta::assert_snapshot!(
            DisplayUnit::Nanometers.format_pair(1.5e-6, 2.0e-6),
            @"(1500.00, 2000.00) nm"
        );
    }

    #[test]
    fn display_unit_fixed_micrometers() {
        insta::assert_snapshot!(
            DisplayUnit::Micrometers.format_pair(5e-8, 1e-7),
            @"(0.050, 0.100) µm"
        );
    }

    #[test]
    fn display_unit_fixed_millimeters() {
        insta::assert_snapshot!(
            DisplayUnit::Millimeters.format_pair(5e-8, 1e-7),
            @"(0.0000, 0.0001) mm"
        );
    }

    #[test]
    fn display_unit_auto_zero() {
        insta::assert_snapshot!(
            DisplayUnit::Auto.format_pair(0.0, 0.0),
            @"(0.00, 0.00) nm"
        );
    }

    #[test]
    fn display_unit_auto_uses_larger_axis_for_scale() {
        // x is in nm range but y is in µm range, so both should display as µm
        insta::assert_snapshot!(
            DisplayUnit::Auto.format_pair(5e-8, 2.0e-6),
            @"(0.050, 2.000) µm"
        );
    }

    #[test]
    fn display_unit_label() {
        insta::assert_snapshot!(DisplayUnit::Auto.label(), @"Auto");
        insta::assert_snapshot!(DisplayUnit::Nanometers.label(), @"nm");
        insta::assert_snapshot!(DisplayUnit::Micrometers.label(), @"µm");
        insta::assert_snapshot!(DisplayUnit::Millimeters.label(), @"mm");
    }

    #[test]
    fn display_unit_default_is_auto() {
        assert_eq!(DisplayUnit::default(), DisplayUnit::Auto);
    }

    #[test]
    fn grid_spacing_default_is_1x() {
        assert!((GridSpacing::default().multiplier - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn grid_spacing_label_preset() {
        insta::assert_snapshot!(GridSpacing { multiplier: 1.0 }.label(), @"1x");
        insta::assert_snapshot!(GridSpacing { multiplier: 0.5 }.label(), @"0.5x");
        insta::assert_snapshot!(GridSpacing { multiplier: 2.0 }.label(), @"2x");
    }

    #[test]
    fn grid_spacing_label_custom() {
        insta::assert_snapshot!(GridSpacing { multiplier: 3.0 }.label(), @"Custom");
    }
}

/// Controls how world coordinates (meters) are displayed to the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayUnit {
    /// Automatically choose nm/µm/mm based on magnitude.
    #[default]
    Auto,
    Nanometers,
    Micrometers,
    Millimeters,
}

impl DisplayUnit {
    pub const ALL: [Self; 4] = [
        Self::Auto,
        Self::Nanometers,
        Self::Micrometers,
        Self::Millimeters,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Nanometers => "nm",
            Self::Micrometers => "µm",
            Self::Millimeters => "mm",
        }
    }

    /// Formats an (x, y) coordinate pair in the selected unit, using a
    /// consistent unit suffix for both axes.
    pub fn format_pair(self, x: f64, y: f64) -> String {
        match self {
            Self::Auto => {
                let abs = x.abs().max(y.abs());
                if abs < 1e-6 {
                    format!("({:.2}, {:.2}) nm", x * 1e9, y * 1e9)
                } else if abs < 1e-3 {
                    format!("({:.3}, {:.3}) µm", x * 1e6, y * 1e6)
                } else {
                    format!("({:.4}, {:.4}) mm", x * 1e3, y * 1e3)
                }
            }
            Self::Nanometers => format!("({:.2}, {:.2}) nm", x * 1e9, y * 1e9),
            Self::Micrometers => format!("({:.3}, {:.3}) µm", x * 1e6, y * 1e6),
            Self::Millimeters => format!("({:.4}, {:.4}) mm", x * 1e3, y * 1e3),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidePanelTab {
    #[default]
    Cells,
    Layers,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CellViewMode {
    #[default]
    Tree,
    Flat,
}

/// Controls grid line spacing as a multiplier on the auto-calculated spacing.
///
/// A multiplier of 1.0 gives the default auto spacing. Smaller values produce
/// a denser grid; larger values produce a sparser grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridSpacing {
    pub multiplier: f64,
}

impl Default for GridSpacing {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

impl GridSpacing {
    pub const PRESETS: &[(&str, f64)] = &[
        ("1x", 1.0),
        ("0.5x", 0.5),
        ("0.25x", 0.25),
        ("0.1x", 0.1),
        ("2x", 2.0),
        ("5x", 5.0),
    ];

    pub fn label(self) -> &'static str {
        for &(label, multiplier) in Self::PRESETS {
            if (self.multiplier - multiplier).abs() < f64::EPSILON {
                return label;
            }
        }
        "Custom"
    }
}

/// Groups layer visibility and color state.
#[derive(Default)]
pub struct LayerState {
    pub layer_colors: LayerColorMap,
    pub hidden_layers: HashSet<(Layer, DataType)>,
    pub filter: String,
}
