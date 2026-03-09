use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;

use egui::{Mesh, Pos2, Shape};
use emath::{TSTransform, Vec2};
use gdsr::{CellStats, DataType, Element, Layer, Library};

use crate::colors::LayerColorMap;
use crate::hierarchy::{self, CellTreeNode, ExpandState};
use crate::spatial::SpatialGrid;

/// Tracks an in-flight file-open operation.
#[derive(Default)]
pub struct FileLoadState {
    pub file_path: Option<PathBuf>,
    pub load_receiver: Option<(PathBuf, mpsc::Receiver<Result<Library, String>>)>,
    pub loading: bool,
    pub error_message: Option<String>,
}

/// Holds the loaded library, selected cell, and its streamed elements.
pub struct CellState {
    pub library: Library,
    pub cell_names: Vec<String>,
    pub cell_tree: Vec<CellTreeNode>,
    pub flat_tree: Vec<CellTreeNode>,
    pub expand_state: ExpandState,
    pub selected_cell: Option<String>,
    pub elements: Vec<Element>,
    pub element_receiver: Option<mpsc::Receiver<Element>>,
    pub elements_loading: bool,
    pub layers: BTreeSet<(Layer, DataType)>,
    pub spatial_grid: Option<SpatialGrid>,
    pub tessellation_cache: HashMap<u32, Vec<usize>>,
    pub cell_stats: Option<CellStats>,
}

impl CellState {
    pub fn new(library: Library) -> Self {
        let mut cell_names: Vec<String> = library.cells().keys().cloned().collect();
        cell_names.sort();
        let cell_tree = hierarchy::build_cell_tree(&library);
        let flat_tree = hierarchy::build_flat_cell_tree(&library);
        let expand_state = ExpandState::default();
        Self {
            library,
            cell_names,
            cell_tree,
            flat_tree,
            expand_state,
            selected_cell: None,
            elements: Vec::new(),
            element_receiver: None,
            elements_loading: false,
            layers: BTreeSet::new(),
            spatial_grid: None,
            tessellation_cache: HashMap::new(),
            cell_stats: None,
        }
    }
}

/// Caches rendered geometry and supports delta-transforms on pan/zoom instead of
/// full re-renders. A full render queries a 3× viewport so there is margin for
/// panning before the cache must be rebuilt.
///
/// Geometry is split into batched layer meshes (few large meshes, cheap to clone)
/// and extra shapes (text, rect fallbacks).
pub struct RenderCache {
    layer_meshes: Vec<((Layer, DataType), Mesh)>,
    extra_shapes: Vec<Shape>,
    /// Viewport state at the time shapes were rendered.
    render_center_x: f64,
    render_center_y: f64,
    render_zoom: f64,
    render_rect_center: Pos2,
    /// Invalidation metadata — if any of these change, full re-render.
    hidden_layers: Vec<(Layer, DataType)>,
    element_count: usize,
    populated: bool,
}

impl Default for RenderCache {
    fn default() -> Self {
        Self {
            layer_meshes: Vec::new(),
            extra_shapes: Vec::new(),
            render_center_x: 0.0,
            render_center_y: 0.0,
            render_zoom: 1.0,
            render_rect_center: Pos2::ZERO,
            hidden_layers: Vec::new(),
            element_count: 0,
            populated: false,
        }
    }
}

impl RenderCache {
    /// Returns `true` when a full re-render is needed (cannot use delta transform).
    pub fn needs_full_render(
        &self,
        hidden_layers: &[(Layer, DataType)],
        element_count: usize,
        current_center_x: f64,
        current_center_y: f64,
        current_zoom: f64,
        rect: egui::Rect,
    ) -> bool {
        if !self.populated {
            return true;
        }
        if self.hidden_layers != hidden_layers || self.element_count != element_count {
            return true;
        }

        // Zoom ratio outside [0.5, 2.0] means LOAD thresholds may have crossed.
        let zoom_ratio = current_zoom / self.render_zoom;
        if !(0.5..=2.0).contains(&zoom_ratio) {
            return true;
        }

        // Pan distance in screen pixels. The margin budget is one full viewport
        // width/height (from the 3× query region). Trigger re-render at 80%.
        let dx_screen = (self.render_center_x - current_center_x) * current_zoom;
        let dy_screen = (self.render_center_y - current_center_y) * current_zoom;
        let margin_x = f64::from(rect.width()) * 0.8;
        let margin_y = f64::from(rect.height()) * 0.8;
        if dx_screen.abs() > margin_x || dy_screen.abs() > margin_y {
            return true;
        }

        false
    }

    /// Computes the `TSTransform` that maps render-time screen coordinates to
    /// current screen coordinates.
    pub fn delta_transform(
        &self,
        current_center_x: f64,
        current_center_y: f64,
        current_zoom: f64,
        rect_center: Pos2,
    ) -> TSTransform {
        let scale = (current_zoom / self.render_zoom) as f32;
        let tx = f64::from(rect_center.x) * f64::from(1.0 - scale)
            + (self.render_center_x - current_center_x) * current_zoom;
        let ty = f64::from(rect_center.y) * f64::from(1.0 - scale)
            - (self.render_center_y - current_center_y) * current_zoom;
        TSTransform::new(Vec2::new(tx as f32, ty as f32), scale)
    }

    /// Stores batched layer meshes, extra shapes, and the viewport state.
    pub fn update(
        &mut self,
        layer_meshes: Vec<((Layer, DataType), Mesh)>,
        extra_shapes: Vec<Shape>,
        center_x: f64,
        center_y: f64,
        zoom: f64,
        rect_center: Pos2,
        hidden_layers: Vec<(Layer, DataType)>,
        element_count: usize,
    ) {
        self.layer_meshes = layer_meshes;
        self.extra_shapes = extra_shapes;
        self.render_center_x = center_x;
        self.render_center_y = center_y;
        self.render_zoom = zoom;
        self.render_rect_center = rect_center;
        self.hidden_layers = hidden_layers;
        self.element_count = element_count;
        self.populated = true;
    }

    pub fn clear(&mut self) {
        self.populated = false;
    }

    pub fn layer_meshes(&self) -> &[((Layer, DataType), Mesh)] {
        &self.layer_meshes
    }

    pub fn extra_shapes(&self) -> &[Shape] {
        &self.extra_shapes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Rect;

    fn test_rect() -> Rect {
        Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(800.0, 600.0))
    }

    fn populated_cache() -> RenderCache {
        let mut cache = RenderCache::default();
        cache.update(
            vec![],
            vec![],
            0.0,
            0.0,
            1.0,
            test_rect().center(),
            vec![],
            42,
        );
        cache
    }

    #[test]
    fn no_rerender_on_same_state() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(!cache.needs_full_render(&[], 42, 0.0, 0.0, 1.0, rect));
    }

    #[test]
    fn rerender_when_empty() {
        let cache = RenderCache::default();
        assert!(cache.needs_full_render(&[], 42, 0.0, 0.0, 1.0, test_rect()));
    }

    #[test]
    fn no_rerender_on_small_pan() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(!cache.needs_full_render(&[], 42, 100.0, 0.0, 1.0, rect));
    }

    #[test]
    fn rerender_on_large_pan() {
        let cache = populated_cache();
        let rect = test_rect();
        // 800px viewport width, margin budget = 800, 80% = 640px.
        // dx_screen = (0.0 - 700.0) * 1.0 = -700, |700| > 640 → re-render
        assert!(cache.needs_full_render(&[], 42, 700.0, 0.0, 1.0, rect));
    }

    #[test]
    fn no_rerender_on_small_zoom() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(!cache.needs_full_render(&[], 42, 0.0, 0.0, 1.5, rect));
    }

    #[test]
    fn rerender_on_large_zoom() {
        let cache = populated_cache();
        let rect = test_rect();
        // zoom ratio 3.0 / 1.0 = 3.0, outside [0.5, 2.0]
        assert!(cache.needs_full_render(&[], 42, 0.0, 0.0, 3.0, rect));
    }

    #[test]
    fn rerender_on_hidden_layer_change() {
        let cache = populated_cache();
        assert!(cache.needs_full_render(
            &[(Layer::new(1), DataType::new(0))],
            42,
            0.0,
            0.0,
            1.0,
            test_rect()
        ));
    }

    #[test]
    fn rerender_on_element_count_change() {
        let cache = populated_cache();
        assert!(cache.needs_full_render(&[], 43, 0.0, 0.0, 1.0, test_rect()));
    }

    #[test]
    fn delta_transform_identity_on_same_state() {
        let cache = populated_cache();
        let rect = test_rect();
        let tsf = cache.delta_transform(0.0, 0.0, 1.0, rect.center());
        assert!((tsf.scaling - 1.0).abs() < 1e-6);
        assert!(tsf.translation.x.abs() < 1e-3);
        assert!(tsf.translation.y.abs() < 1e-3);
    }

    #[test]
    fn delta_transform_pure_pan() {
        let cache = populated_cache();
        let rect = test_rect();
        let zoom = 1.0;
        let pan_x = 50.0;
        let tsf = cache.delta_transform(pan_x, 0.0, zoom, rect.center());

        // Pure pan: scale=1, tx = (0 - 50) * 1 = -50, ty = 0
        assert!((tsf.scaling - 1.0).abs() < 1e-6);
        assert!((tsf.translation.x - (-50.0)).abs() < 1e-3);
        assert!(tsf.translation.y.abs() < 1e-3);
    }

    /// Verifies the delta transform maps an old screen point to where it should be
    /// after a viewport change, matching a fresh `world_to_screen` call.
    #[test]
    fn delta_transform_matches_fresh_render() {
        use crate::viewport::Viewport;

        let rect = test_rect();
        let old_vp = Viewport {
            center_x: 10.0,
            center_y: 20.0,
            zoom: 100.0,
        };

        let mut cache = RenderCache::default();
        cache.update(
            vec![],
            vec![],
            old_vp.center_x,
            old_vp.center_y,
            old_vp.zoom,
            rect.center(),
            vec![],
            1,
        );

        let new_vp = Viewport {
            center_x: 15.0,
            center_y: 25.0,
            zoom: 120.0,
        };

        let tsf =
            cache.delta_transform(new_vp.center_x, new_vp.center_y, new_vp.zoom, rect.center());

        // Pick a world point, render with old viewport, apply delta, compare to new viewport.
        let (wx, wy) = (12.0, 22.0);
        let old_screen = old_vp.world_to_screen(wx, wy, rect);
        let transformed = tsf * old_screen;
        let fresh = new_vp.world_to_screen(wx, wy, rect);

        assert!(
            (transformed.x - fresh.x).abs() < 0.5,
            "x: {transformed} vs {fresh}"
        );
        assert!(
            (transformed.y - fresh.y).abs() < 0.5,
            "y: {transformed} vs {fresh}"
        );
    }

    #[test]
    fn no_rerender_at_zoom_ratio_boundary_low() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(!cache.needs_full_render(&[], 42, 0.0, 0.0, 0.5, rect));
    }

    #[test]
    fn rerender_just_below_zoom_ratio_boundary() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(cache.needs_full_render(&[], 42, 0.0, 0.0, 0.49, rect));
    }

    #[test]
    fn no_rerender_at_zoom_ratio_boundary_high() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(!cache.needs_full_render(&[], 42, 0.0, 0.0, 2.0, rect));
    }

    #[test]
    fn rerender_just_above_zoom_ratio_boundary() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(cache.needs_full_render(&[], 42, 0.0, 0.0, 2.01, rect));
    }

    #[test]
    fn no_rerender_at_exact_margin() {
        let cache = populated_cache();
        let rect = test_rect();
        // margin_x = 800 * 0.8 = 640, dx_screen = |0 - 640| = 640, NOT > 640
        assert!(!cache.needs_full_render(&[], 42, 640.0, 0.0, 1.0, rect));
    }

    #[test]
    fn rerender_just_beyond_margin() {
        let cache = populated_cache();
        let rect = test_rect();
        assert!(cache.needs_full_render(&[], 42, 641.0, 0.0, 1.0, rect));
    }

    #[test]
    fn delta_transform_extreme_zoom_ratio() {
        let cache = populated_cache();
        let rect = test_rect();
        let tsf = cache.delta_transform(0.0, 0.0, 2.0, rect.center());
        assert!(tsf.scaling.is_finite());
        assert!(tsf.translation.x.is_finite());
        assert!(tsf.translation.y.is_finite());
    }

    #[test]
    fn delta_transform_large_pan() {
        let cache = populated_cache();
        let rect = test_rect();
        let tsf = cache.delta_transform(1e6, 1e6, 1.0, rect.center());
        assert!(tsf.translation.x.is_finite());
        assert!(tsf.translation.y.is_finite());
    }

    #[test]
    fn clear_forces_rerender() {
        let mut cache = populated_cache();
        let rect = test_rect();
        assert!(!cache.needs_full_render(&[], 42, 0.0, 0.0, 1.0, rect));
        cache.clear();
        assert!(cache.needs_full_render(&[], 42, 0.0, 0.0, 1.0, rect));
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
}
