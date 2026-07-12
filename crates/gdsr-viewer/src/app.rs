use std::path::{Path, PathBuf};
use std::sync::mpsc;

use gdsr::{DEFAULT_FLOAT_UNITS, DEFAULT_INTEGER_UNITS, Point};

use crate::drawable::{Drawable, cell_world_bbox};
use crate::panels;
use crate::quick_pick::{QuickPick, QuickPickResult};
use crate::recent::{RecentProjectItem, RecentProjects};
use crate::ruler::RulerState;
use crate::spatial::SpatialGrid;
use crate::state::{
    CellState, CellViewMode, DisplayUnit, FileLoadState, GridSpacing, LayerState, RenderCache,
    SidePanelTab,
};
use crate::viewport::Viewport;

/// Returns shortcut text with the platform-appropriate modifier (⌘ on macOS, Ctrl on others).
fn shortcut_text(key: &str) -> String {
    let modifier = if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl+"
    };
    format!("{modifier}{key}")
}

pub struct ViewerApp {
    file_load: FileLoadState,
    cell: Option<CellState>,
    layer_state: LayerState,
    viewport: Viewport,
    mouse_world_pos: Option<(f64, f64)>,
    render_cache: RenderCache,
    ruler: RulerState,
    show_grid: bool,
    hovered_element: Option<usize>,
    selected_element: Option<usize>,
    /// Reusable scratch buffer for spatial grid point queries.
    query_buf: Vec<u32>,
    /// Reusable mark buffer for deduplicating elements across visible spatial cells.
    drawn_element_marks: Vec<bool>,
    side_panel_tab: SidePanelTab,
    cell_view_mode: CellViewMode,
    scroll_to_selected: bool,
    recent_projects: RecentProjects,
    display_unit: DisplayUnit,
    grid_spacing: GridSpacing,
    cell_picker: QuickPick<String>,
    recent_picker: QuickPick<RecentProjectItem>,
    has_unsaved_changes: bool,
    save_error: Option<String>,
    show_unsaved_close_prompt: bool,
}

impl Default for ViewerApp {
    fn default() -> Self {
        Self {
            file_load: FileLoadState::default(),
            cell: None,
            layer_state: LayerState::default(),
            viewport: Viewport::default(),
            mouse_world_pos: None,
            render_cache: RenderCache::default(),
            ruler: RulerState::default(),
            show_grid: true,
            hovered_element: None,
            selected_element: None,
            query_buf: Vec::new(),
            drawn_element_marks: Vec::new(),
            side_panel_tab: SidePanelTab::default(),
            cell_view_mode: CellViewMode::default(),
            scroll_to_selected: false,
            display_unit: DisplayUnit::default(),
            grid_spacing: GridSpacing::default(),
            recent_projects: RecentProjects::load(),
            cell_picker: QuickPick::new("Search cells…", true),
            recent_picker: QuickPick::new("Recent projects…", false),
            has_unsaved_changes: false,
            save_error: None,
            show_unsaved_close_prompt: false,
        }
    }
}

impl ViewerApp {
    pub fn with_path(path: &Path) -> Self {
        let (path, rx) = crate::loader::load_request(path);
        let file_load = FileLoadState {
            file_path: Some(path.clone()),
            load_receiver: Some((path, rx)),
            loading: true,
            error_message: None,
        };

        Self {
            file_load,
            ..Default::default()
        }
    }

    /// Opens a native file dialog and starts loading the selected GDS file on a background thread.
    fn open_file_dialog(&mut self) {
        if let Some((path, rx)) = crate::loader::load_file_dialog() {
            self.file_load.load_receiver = Some((path, rx));
            self.file_load.loading = true;
            self.file_load.error_message = None;
        }
    }

    /// Called when the background file loader completes successfully. Populates cell
    /// names and auto-selects the first cell.
    fn on_library_loaded(&mut self, library: gdsr::Library, path: PathBuf) {
        self.recent_projects.add(&path);
        self.recent_projects.save();

        let cell_state = CellState::new(library);
        self.cell = Some(cell_state);
        self.hovered_element = None;
        self.selected_element = None;
        self.file_load.file_path = Some(path);
        self.file_load.loading = false;
        self.has_unsaved_changes = false;
        self.save_error = None;
        self.show_unsaved_close_prompt = false;
    }

    /// Switches to a new cell, keeping hierarchy intact for draw-time expansion.
    fn select_cell(&mut self, name: &str) {
        if let Some(cell) = self.cell.as_mut() {
            cell.selected_cell = Some(name.to_string());
            cell.expand_state.set_expanded(name, true);
            self.scroll_to_selected = true;
            self.hovered_element = None;
            self.selected_element = None;
            if cell.load_direct_cell_elements(name) {
                for &(layer, data_type) in &cell.layers {
                    self.layer_state.layer_colors.get(layer, data_type);
                }
            }
        }
        self.render_cache.clear();
    }

    /// Adjusts the viewport to fit all currently loaded elements.
    fn zoom_to_fit(&mut self) {
        if let Some(cell) = self.cell.as_ref() {
            let bounds = cell
                .selected_cell
                .as_ref()
                .and_then(|name| cell_world_bbox(name, &cell.library));
            if let Some(bounds) = bounds {
                let rect =
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(800.0, 600.0));
                self.viewport.zoom_to_fit(&bounds, rect);
            }
        }
    }

    /// Loads a file from a path (used by recent projects).
    fn load_path(&mut self, path: &Path) {
        let (path, rx) = crate::loader::load_request(path);
        self.file_load.load_receiver = Some((path, rx));
        self.file_load.loading = true;
        self.file_load.error_message = None;
        self.save_error = None;
    }

    fn save_current_file(&mut self) -> bool {
        if self.cell.is_none() {
            self.save_error = Some("No GDS library loaded".to_string());
            return false;
        }

        if let Some(path) = self.file_load.file_path.clone() {
            return self.save_file_to_path(&path);
        }

        self.save_file_as_dialog()
    }

    fn save_file_as_dialog(&mut self) -> bool {
        if self.cell.is_none() {
            self.save_error = Some("No GDS library loaded".to_string());
            return false;
        }

        let mut dialog = rfd::FileDialog::new().add_filter("GDS files", &["gds", "gds2", "gdsii"]);

        if let Some(path) = &self.file_load.file_path {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                dialog = dialog.set_file_name(file_name);
            }
        } else {
            dialog = dialog.set_file_name("layout.gds");
        }

        let Some(path) = dialog.save_file() else {
            return false;
        };

        self.save_file_to_path(&path)
    }

    fn save_file_to_path(&mut self, path: &Path) -> bool {
        let Some(cell) = self.cell.as_ref() else {
            self.save_error = Some("No GDS library loaded".to_string());
            return false;
        };

        match cell
            .library
            .write_file(path, DEFAULT_FLOAT_UNITS, DEFAULT_INTEGER_UNITS)
        {
            Ok(()) => {
                self.file_load.file_path = Some(path.to_path_buf());
                self.has_unsaved_changes = false;
                self.save_error = None;
                self.show_unsaved_close_prompt = false;
                self.recent_projects.add(path);
                self.recent_projects.save();
                true
            }
            Err(err) => {
                self.save_error = Some(err.to_string());
                false
            }
        }
    }

    fn cancel_close_for_unsaved_changes(&mut self) -> bool {
        if !self.has_unsaved_changes {
            return false;
        }

        self.show_unsaved_close_prompt = true;
        true
    }

    fn delete_selected_element(&mut self) -> bool {
        let Some(index) = self.selected_element else {
            return false;
        };
        let Some(cell) = self.cell.as_mut() else {
            self.selected_element = None;
            return false;
        };
        if !cell.delete_element(index) {
            self.selected_element = None;
            return false;
        }

        self.selected_element = None;
        self.hovered_element = None;
        self.render_cache.clear();
        self.has_unsaved_changes = true;
        self.save_error = None;
        true
    }

    fn move_selected_element(&mut self, delta: Point) -> bool {
        let Some(index) = self.selected_element else {
            return false;
        };
        let Some(cell) = self.cell.as_mut() else {
            self.selected_element = None;
            return false;
        };
        if !cell.move_element(index, delta) {
            self.selected_element = None;
            return false;
        }

        self.hovered_element = Some(index);
        self.render_cache.clear();
        self.has_unsaved_changes = true;
        self.save_error = None;
        true
    }

    fn draw_unsaved_close_prompt(&mut self, ctx: &egui::Context) {
        if !self.show_unsaved_close_prompt {
            return;
        }

        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label("Save changes before closing?");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() && self.save_current_file() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Discard").clicked() {
                        self.has_unsaved_changes = false;
                        self.show_unsaved_close_prompt = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_unsaved_close_prompt = false;
                    }
                });
            });
    }
}

fn hit_test_element(
    elements: &[gdsr::Element],
    grid: &SpatialGrid,
    query_buf: &mut Vec<u32>,
    wx: f64,
    wy: f64,
    zoom: f64,
) -> Option<usize> {
    let candidates = grid.query_point(wx, wy, query_buf);
    for &idx in candidates.iter().rev() {
        if let Some(el) = elements.get(idx as usize) {
            if el.hit_test(wx, wy, zoom) {
                return Some(idx as usize);
            }
        }
    }
    None
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && self.cancel_close_for_unsaved_changes()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // Poll background loader
        if let Some((path, rx)) = self.file_load.load_receiver.take() {
            match rx.try_recv() {
                Ok(Ok(library)) => {
                    self.on_library_loaded(library, path);
                }
                Ok(Err(err)) => {
                    self.file_load.error_message = Some(err);
                    self.file_load.loading = false;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.file_load.load_receiver = Some((path, rx));
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.file_load.error_message =
                        Some("File loading thread disconnected".to_string());
                    self.file_load.loading = false;
                }
            }
        }

        if let Some(cell) = self.cell.as_mut() {
            if self
                .selected_element
                .is_some_and(|idx| idx >= cell.elements.len())
            {
                self.selected_element = None;
            }
        }

        // Global keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.zoom_to_fit();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::G)) {
            self.show_grid = !self.show_grid;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O) && !i.modifiers.alt) {
            self.open_file_dialog();
        }
        if ctx.input(|i| i.modifiers.command && i.modifiers.alt && i.key_pressed(egui::Key::O)) {
            self.recent_picker.toggle();
        }
        if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.save_file_as_dialog();
        } else if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save_current_file();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.cell_picker.toggle();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::R)) {
            self.ruler.clear_all();
        } else if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.ruler.toggle();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape))
            && !self.cell_picker.is_open()
            && !self.recent_picker.is_open()
        {
            self.ruler.cancel();
        }
        let delete_pressed =
            ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
        if delete_pressed
            && !self.cell_picker.is_open()
            && !self.recent_picker.is_open()
            && self.delete_selected_element()
        {
            ctx.request_repaint();
        }
        if self.ruler.start.is_some() {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("Open...").shortcut_text(shortcut_text("O")))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.open_file_dialog();
                    }
                    if ui
                        .add(
                            egui::Button::new("Recent Projects...")
                                .shortcut_text(shortcut_text("⌥O")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.recent_picker.open();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.cell.is_some(),
                            egui::Button::new("Save").shortcut_text(shortcut_text("S")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.save_current_file();
                    }
                    if ui
                        .add_enabled(
                            self.cell.is_some(),
                            egui::Button::new("Save As...").shortcut_text(shortcut_text("Shift+S")),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.save_file_as_dialog();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .add(egui::Button::new("Zoom to Fit").shortcut_text("F"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.zoom_to_fit();
                    }
                    if ui
                        .add(egui::Button::new("Zoom In").shortcut_text("+"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.viewport.zoom_at_center(1.2);
                    }
                    if ui
                        .add(egui::Button::new("Zoom Out").shortcut_text("-"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.viewport.zoom_at_center(1.0 / 1.2);
                    }
                    ui.separator();
                    if ui
                        .add(egui::Button::new("Ruler").shortcut_text("R"))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.ruler.toggle();
                    }
                    if ui
                        .add(egui::Button::new("Clear Rulers").shortcut_text(shortcut_text("R")))
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.ruler.clear_all();
                    }
                    if ui
                        .add(
                            egui::Button::new(if self.show_grid {
                                "Hide Grid"
                            } else {
                                "Show Grid"
                            })
                            .shortcut_text("G"),
                        )
                        .clicked()
                    {
                        ui.close_kind(egui::UiKind::Menu);
                        self.show_grid = !self.show_grid;
                    }
                    ui.separator();
                    ui.label("Pan: Arrow Keys");
                });
            });
        });

        // Bottom activity bar
        let mut depth_changed = false;
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let is_tree = self.side_panel_tab == SidePanelTab::Cells
                    && self.cell_view_mode == CellViewMode::Tree;
                if ui
                    .selectable_label(is_tree, "Tree")
                    .on_hover_text("Cell hierarchy")
                    .clicked()
                {
                    self.side_panel_tab = SidePanelTab::Cells;
                    self.cell_view_mode = CellViewMode::Tree;
                }

                let is_flat = self.side_panel_tab == SidePanelTab::Cells
                    && self.cell_view_mode == CellViewMode::Flat;
                if ui
                    .selectable_label(is_flat, "Cells")
                    .on_hover_text("All cells")
                    .clicked()
                {
                    self.side_panel_tab = SidePanelTab::Cells;
                    self.cell_view_mode = CellViewMode::Flat;
                    if self
                        .cell
                        .as_ref()
                        .and_then(|c| c.selected_cell.as_ref())
                        .is_some()
                    {
                        self.scroll_to_selected = true;
                    }
                }

                if ui
                    .selectable_label(self.side_panel_tab == SidePanelTab::Layers, "Layers")
                    .clicked()
                {
                    self.side_panel_tab = SidePanelTab::Layers;
                }

                ui.separator();

                if let Some((wx, wy)) = self.mouse_world_pos {
                    ui.label(self.display_unit.format_pair(wx, wy));
                }

                ui.separator();

                if self.file_load.loading {
                    ui.label("Loading...");
                } else if let Some(err) = &self.file_load.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
                } else if let Some(err) = &self.save_error {
                    ui.colored_label(egui::Color32::RED, format!("Save error: {err}"));
                } else if let Some(path) = &self.file_load.file_path {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    ui.label(if self.has_unsaved_changes {
                        format!("{file_name}*")
                    } else {
                        file_name.to_string()
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("display_unit")
                        .selected_text(self.display_unit.label())
                        .width(60.0)
                        .show_ui(ui, |ui| {
                            for unit in DisplayUnit::ALL {
                                ui.selectable_value(&mut self.display_unit, unit, unit.label());
                            }
                        });
                    ui.label("Grid:");
                    egui::ComboBox::from_id_salt("grid_spacing")
                        .selected_text(self.grid_spacing.label())
                        .width(60.0)
                        .show_ui(ui, |ui| {
                            for &(label, multiplier) in GridSpacing::PRESETS {
                                let preset = GridSpacing { multiplier };
                                ui.selectable_value(&mut self.grid_spacing, preset, label);
                            }
                        });
                    if let Some(cell) = &mut self.cell {
                        let prev_depth = cell.render_depth;
                        if ui.small_button("+").clicked() && cell.render_depth < 99 {
                            cell.render_depth += 1;
                        }
                        ui.label(format!("Depth: {}", cell.render_depth));
                        if ui.small_button("−").clicked() && cell.render_depth > 0 {
                            cell.render_depth -= 1;
                        }
                        if cell.render_depth != prev_depth {
                            depth_changed = true;
                        }
                    }
                    if self.ruler.active {
                        ui.label("Ruler: click to place point (Esc to cancel)");
                    }
                    if let Some(stats) = self.cell.as_ref().and_then(|c| c.cell_stats.as_ref()) {
                        panels::draw_stats_bar(ui, stats);
                    }
                });
            });
        });

        if depth_changed {
            if let Some(name) = self.cell.as_ref().and_then(|c| c.selected_cell.clone()) {
                self.select_cell(&name);
            }
            self.render_cache.clear();
        }

        // Cell picker (⌘P)
        self.cell_picker.set_items(
            self.cell
                .as_ref()
                .map(|c| c.cell_names.clone())
                .unwrap_or_default(),
        );
        if let QuickPickResult::Selected(idx) = self.cell_picker.show(ctx) {
            let name = self.cell_picker.items()[idx].clone();
            self.select_cell(&name);
        }

        // Recent projects picker (⌘⌥O)
        self.recent_picker.set_items(
            self.recent_projects
                .paths()
                .iter()
                .map(|p| RecentProjectItem::from_path(p))
                .collect(),
        );
        if let QuickPickResult::Selected(idx) = self.recent_picker.show(ctx) {
            let path = self.recent_picker.items()[idx].path.clone();
            self.load_path(&path);
        }

        let mut cell_changed = false;
        let mut color_changed = false;
        let cell = &mut self.cell;
        let layer_state = &mut self.layer_state;
        let active_tab = self.side_panel_tab;
        let view_mode = self.cell_view_mode;
        let scroll_to_selected = &mut self.scroll_to_selected;
        let selected_element_idx = self.selected_element;
        let mut delete_selected = false;
        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .width_range(40.0..=800.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.allocate_at_least(egui::vec2(ui.available_width(), 0.0), egui::Sense::hover());
                if let Some(cell) = cell.as_mut() {
                    let selected_element =
                        selected_element_idx.and_then(|idx| cell.elements.get(idx));
                    delete_selected = panels::draw_side_panel(
                        ui,
                        active_tab,
                        &cell.cell_tree,
                        &cell.flat_tree,
                        view_mode,
                        &mut cell.selected_cell,
                        &mut cell_changed,
                        &mut color_changed,
                        &mut cell.expand_state,
                        scroll_to_selected,
                        &cell.layers,
                        layer_state,
                        selected_element,
                    );
                }
            });

        if delete_selected && self.delete_selected_element() {
            ctx.request_repaint();
        }

        if cell_changed {
            if let Some(name) = self.cell.as_ref().and_then(|c| c.selected_cell.clone()) {
                self.select_cell(&name);
            }
        }

        // Clearing render cache after changing color fixes the bug when color is not updated until user zooms out and back in.
        if color_changed {
            self.render_cache.clear();
        }

        let cell = &mut self.cell;
        let viewport = &mut self.viewport;
        let layer_state = &mut self.layer_state;
        let mouse_world_pos = &mut self.mouse_world_pos;
        let render_cache = &mut self.render_cache;
        let ruler = &mut self.ruler;
        let show_grid = self.show_grid;
        let grid_spacing = self.grid_spacing;
        let hovered_element = &mut self.hovered_element;
        let selected_element = &mut self.selected_element;
        let query_buf = &mut self.query_buf;
        let drawn_element_marks = &mut self.drawn_element_marks;
        let render_depth = cell.as_ref().map_or(1, |c| c.render_depth);
        let selected_cell_name: Option<String> =
            cell.as_ref().and_then(|c| c.selected_cell.clone());
        let mut selected_element_drag_delta = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut empty_cache = std::collections::HashMap::new();
            let (elements, spatial_grid, library, tessellation_cache) =
                if let Some(cell) = cell.as_mut() {
                    (
                        cell.elements.as_slice(),
                        cell.spatial_grid.as_ref(),
                        Some(&cell.library),
                        &mut cell.tessellation_cache,
                    )
                } else {
                    (&[] as &[gdsr::Element], None, None, &mut empty_cache)
                };

            let interaction = viewport.draw(
                ui,
                elements,
                layer_state,
                spatial_grid,
                query_buf,
                drawn_element_marks,
                library,
                render_cache,
                tessellation_cache,
                ruler,
                show_grid,
                grid_spacing,
                *hovered_element,
                *selected_element,
                render_depth,
                selected_cell_name.as_deref(),
            );
            selected_element_drag_delta = interaction.selected_element_drag_delta;
            *mouse_world_pos = interaction.mouse_world;

            let prev_hovered = *hovered_element;
            let prev_selected = *selected_element;
            *hovered_element = None;
            if let Some((wx, wy)) = *mouse_world_pos {
                if let Some(grid) = spatial_grid {
                    *hovered_element =
                        hit_test_element(elements, grid, query_buf, wx, wy, viewport.zoom);
                }
            }
            if interaction.clicked {
                *selected_element = *hovered_element;
            }
            if *hovered_element != prev_hovered || *selected_element != prev_selected {
                ctx.request_repaint();
            }
        });
        if let Some((dx, dy)) = selected_element_drag_delta {
            if self.move_selected_element(Point::float(dx, dy, 1.0)) {
                ctx.request_repaint();
            }
        }

        self.draw_unsaved_close_prompt(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdsr::{Cell, DataType, Element, Layer, Library, Point, Polygon, Reference};

    fn p(x: f64, y: f64) -> Point {
        Point::float(x, y, 1.0)
    }

    fn test_elements() -> Vec<Element> {
        vec![
            Polygon::new(
                [p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
                Layer::new(1),
                DataType::new(0),
            )
            .into(),
        ]
    }

    fn cell_state_with_test_elements() -> CellState {
        let mut source_cell = Cell::new("top");
        for element in test_elements() {
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
    fn hit_test_element_returns_matching_index() {
        let elements = test_elements();
        let Some(bounds) = crate::viewport::compute_bounds(&elements) else {
            panic!("test elements should have bounds");
        };
        let grid = SpatialGrid::build(&elements, &bounds);
        let mut query_buf = Vec::new();

        assert_eq!(
            hit_test_element(&elements, &grid, &mut query_buf, 5.0, 5.0, 1.0),
            Some(0)
        );
    }

    #[test]
    fn hit_test_element_returns_none_for_miss() {
        let elements = test_elements();
        let Some(bounds) = crate::viewport::compute_bounds(&elements) else {
            panic!("test elements should have bounds");
        };
        let grid = SpatialGrid::build(&elements, &bounds);
        let mut query_buf = Vec::new();

        assert_eq!(
            hit_test_element(&elements, &grid, &mut query_buf, 20.0, 20.0, 1.0),
            None
        );
    }

    #[test]
    fn delete_selected_element_removes_element_marks_dirty_and_clears_selection() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            hovered_element: Some(0),
            ..Default::default()
        };

        assert!(app.delete_selected_element());

        let cell = app.cell.expect("cell should remain loaded");
        assert!(cell.elements.is_empty());
        assert!(
            cell.library
                .get_cell("top")
                .expect("top cell should exist")
                .elements()
                .is_empty()
        );
        assert!(cell.spatial_grid.is_none());
        assert!(app.selected_element.is_none());
        assert!(app.hovered_element.is_none());
        assert!(app.has_unsaved_changes);
    }

    #[test]
    fn move_selected_element_moves_element_marks_dirty_and_keeps_selection() {
        let cell = cell_state_with_test_elements();
        let mut app = ViewerApp {
            cell: Some(cell),
            selected_element: Some(0),
            ..Default::default()
        };

        assert!(app.move_selected_element(Point::float(2.0, 3.0, 1.0)));

        let cell = app.cell.expect("cell should remain loaded");
        let bbox = cell.elements[0]
            .world_bbox()
            .expect("moved element has bbox");
        assert!((bbox.min_x - 2.0).abs() < 1e-12);
        assert!((bbox.min_y - 3.0).abs() < 1e-12);
        assert_eq!(app.selected_element, Some(0));
        assert_eq!(app.hovered_element, Some(0));
        assert!(cell.spatial_grid.is_some());
        assert!(app.has_unsaved_changes);

        let library_bbox = cell
            .library
            .get_cell("top")
            .expect("top cell should exist")
            .elements()[0]
            .world_bbox()
            .expect("moved element has bbox");
        assert!((library_bbox.min_x - 2.0).abs() < 1e-12);
    }

    #[test]
    fn save_file_to_path_writes_library_and_clears_dirty_state() {
        let dir = tempfile::tempdir().expect("temporary directory should be created");
        let path = dir.path().join("saved.gds");
        let mut app = ViewerApp {
            cell: Some(cell_state_with_test_elements()),
            has_unsaved_changes: true,
            ..Default::default()
        };

        assert!(app.save_file_to_path(&path));

        assert!(!app.has_unsaved_changes);
        assert!(app.save_error.is_none());
        assert_eq!(app.file_load.file_path.as_deref(), Some(path.as_path()));

        let saved = Library::read_file(&path, Some(DEFAULT_INTEGER_UNITS))
            .expect("saved GDS should be readable");
        assert_eq!(
            saved
                .get_cell("top")
                .expect("top cell should exist")
                .elements()
                .len(),
            1
        );
    }

    #[test]
    fn save_current_file_without_loaded_library_sets_error() {
        let mut app = ViewerApp::default();

        assert!(!app.save_current_file());
        assert_eq!(app.save_error.as_deref(), Some("No GDS library loaded"));
    }

    #[test]
    fn dirty_close_request_opens_prompt_and_cancels_close() {
        let mut app = ViewerApp {
            has_unsaved_changes: true,
            ..Default::default()
        };

        assert!(app.cancel_close_for_unsaved_changes());
        assert!(app.show_unsaved_close_prompt);
    }

    #[test]
    fn clean_close_request_does_not_open_prompt() {
        let mut app = ViewerApp::default();

        assert!(!app.cancel_close_for_unsaved_changes());
        assert!(!app.show_unsaved_close_prompt);
    }

    #[test]
    fn select_cell_loads_direct_elements_without_flattening_references() {
        let mut leaf = Cell::new("leaf");
        leaf.add(test_elements().remove(0));

        let mut top = Cell::new("top");
        top.add(Reference::new("leaf"));

        let mut library = Library::new("test");
        library.add_cell(leaf);
        library.add_cell(top);

        let mut cell_state = CellState::new(library);
        cell_state.render_depth = 8;
        let mut app = ViewerApp {
            cell: Some(cell_state),
            ..Default::default()
        };

        app.select_cell("top");

        let cell = app.cell.expect("cell should be loaded");
        assert_eq!(cell.elements.len(), 1);
        assert!(matches!(cell.elements[0], Element::Reference(_)));
        assert_eq!(
            cell.layers,
            std::collections::BTreeSet::from([(Layer::new(1), DataType::new(0))])
        );
    }
}
