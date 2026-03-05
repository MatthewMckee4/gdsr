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
        let mut expand_state = ExpandState::default();
        expand_state.expand_all(&cell_tree);
        Self {
            library,
            cell_names,
            cell_tree,
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
}

/// Groups layer visibility and color state.
#[derive(Default)]
pub struct LayerState {
    pub layer_colors: LayerColorMap,
    pub hidden_layers: HashSet<(Layer, DataType)>,
}
