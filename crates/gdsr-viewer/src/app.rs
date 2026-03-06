use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use crate::drawable::Drawable;
use crate::panels;
use crate::quick_pick::{QuickPick, QuickPickResult};
use crate::recent::RecentProjects;
use crate::ruler::RulerState;
use crate::spatial::SpatialGrid;
use crate::state::{
    CellState, CellViewMode, DisplayUnit, FileLoadState, LayerState, RenderCache, SidePanelTab,
};
use crate::viewport::{self, Viewport};

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
    /// Reusable scratch buffer for spatial grid point queries.
    query_buf: Vec<u32>,
    side_panel_tab: SidePanelTab,
    cell_view_mode: CellViewMode,
    scroll_to_selected: bool,
    recent_projects: RecentProjects,
    display_unit: DisplayUnit,
    cell_picker: QuickPick,
    recent_picker: QuickPick,
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
            query_buf: Vec::new(),
            side_panel_tab: SidePanelTab::default(),
            cell_view_mode: CellViewMode::default(),
            scroll_to_selected: false,
            display_unit: DisplayUnit::default(),
            recent_projects: RecentProjects::load(),
            cell_picker: QuickPick::new("Search cells…"),
            recent_picker: QuickPick::new("Recent projects…"),
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
        self.file_load.file_path = Some(path);
        self.file_load.loading = false;
    }

    /// Switches to a new cell, cancelling any in-flight element streaming and starting
    /// a new streaming thread for the selected cell's elements.
    fn select_cell(&mut self, name: &str) {
        if let Some(cell) = self.cell.as_mut() {
            cell.selected_cell = Some(name.to_string());
            cell.expand_state.set_expanded(name, true);
            self.scroll_to_selected = true;
            cell.element_receiver = None;
            cell.elements.clear();
            cell.layers.clear();
            cell.spatial_grid = None;
            cell.cell_stats = cell.library.get_cell(name).map(gdsr::CellStats::from_cell);

            if let Some(cell_data) = cell.library.get_cell(name) {
                let cell_data = cell_data.clone();
                let library = cell.library.clone();
                let (tx, rx) = mpsc::channel();

                thread::spawn(move || {
                    cell_data.stream_elements(None, &library, &tx);
                });

                cell.element_receiver = Some(rx);
                cell.elements_loading = true;
            }
        }
    }

    /// Adjusts the viewport to fit all currently loaded elements.
    fn zoom_to_fit(&mut self) {
        if let Some(cell) = self.cell.as_ref() {
            if let Some(bounds) = viewport::compute_bounds(&cell.elements) {
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
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        // Drain element streaming channel
        let mut streaming_finished = false;
        if let Some(cell) = self.cell.as_mut() {
            if let Some(rx) = &cell.element_receiver {
                loop {
                    match rx.try_recv() {
                        Ok(element) => {
                            for key in element.layer_keys() {
                                if cell.layers.insert(key) {
                                    self.layer_state.layer_colors.get(key.0, key.1);
                                }
                            }
                            cell.elements.push(element);
                        }
                        Err(mpsc::TryRecvError::Empty) => break,
                        Err(mpsc::TryRecvError::Disconnected) => {
                            cell.elements_loading = false;
                            cell.element_receiver = None;
                            if let Some(bounds) = viewport::compute_bounds(&cell.elements) {
                                cell.spatial_grid =
                                    Some(SpatialGrid::build(&cell.elements, &bounds));
                            }
                            streaming_finished = true;
                            break;
                        }
                    }
                }
            }

            if cell.elements_loading {
                ctx.request_repaint();
            }
        }

        if streaming_finished {
            self.zoom_to_fit();
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
                } else if self.cell.as_ref().is_some_and(|c| c.elements_loading) {
                    ui.label(format!(
                        "Expanding elements... ({})",
                        self.cell.as_ref().map_or(0, |c| c.elements.len())
                    ));
                } else if let Some(err) = &self.file_load.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
                } else if let Some(path) = &self.file_load.file_path {
                    ui.label(
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown"),
                    );
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
                    if self.ruler.active {
                        ui.label("Ruler: click to place point (Esc to cancel)");
                    }
                    if let Some(stats) = self.cell.as_ref().and_then(|c| c.cell_stats.as_ref()) {
                        panels::draw_stats_bar(ui, stats);
                    }
                });
            });
        });

        // Cell picker (⌘P)
        let cell_names: Vec<String> = self
            .cell
            .as_ref()
            .map(|c| c.cell_names.clone())
            .unwrap_or_default();
        if let QuickPickResult::Selected(idx) = self.cell_picker.show(ctx, &cell_names, true) {
            self.select_cell(&cell_names[idx]);
        }

        // Recent projects picker (⌘⌥O)
        let recent_labels: Vec<String> = self
            .recent_projects
            .paths()
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        if let QuickPickResult::Selected(idx) = self.recent_picker.show(ctx, &recent_labels, false)
        {
            let path = self.recent_projects.paths()[idx].clone();
            self.load_path(&path);
        }

        let mut cell_changed = false;
        let mut color_changed = false;
        let cell = &mut self.cell;
        let layer_state = &mut self.layer_state;
        let active_tab = self.side_panel_tab;
        let view_mode = self.cell_view_mode;
        let scroll_to_selected = &mut self.scroll_to_selected;
        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .width_range(40.0..=800.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.allocate_at_least(egui::vec2(ui.available_width(), 0.0), egui::Sense::hover());
                if let Some(cell) = cell.as_mut() {
                    panels::draw_side_panel(
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
                    );
                }
            });

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
        let hovered_element = &mut self.hovered_element;
        let query_buf = &mut self.query_buf;
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

            *mouse_world_pos = viewport.draw(
                ui,
                elements,
                layer_state,
                spatial_grid,
                library,
                render_cache,
                tessellation_cache,
                ruler,
                show_grid,
                *hovered_element,
            );

            let prev_hovered = *hovered_element;
            *hovered_element = None;
            if let Some((wx, wy)) = *mouse_world_pos {
                if let Some(grid) = spatial_grid {
                    let candidates = grid.query_point(wx, wy, query_buf);
                    for &idx in candidates.iter().rev() {
                        if let Some(el) = elements.get(idx as usize) {
                            if el.hit_test(wx, wy, viewport.zoom) {
                                *hovered_element = Some(idx as usize);
                                break;
                            }
                        }
                    }
                }
            }
            if *hovered_element != prev_hovered {
                ctx.request_repaint();
            }
        });
    }
}
