use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use crate::drawable::Drawable;
use crate::panels;
use crate::spatial::SpatialGrid;
use crate::state::{CellState, FileLoadState, LayerState, RenderCache};
use crate::viewport::{self, Viewport};

#[derive(Default)]
pub struct ViewerApp {
    file_load: FileLoadState,
    cell: Option<CellState>,
    layer_state: LayerState,
    viewport: Viewport,
    mouse_world_pos: Option<(f64, f64)>,
    render_cache: RenderCache,
}

impl ViewerApp {
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
        let cell_state = CellState::new(library);
        let first_cell = cell_state.cell_names.first().cloned();
        self.cell = Some(cell_state);
        self.file_load.file_path = Some(path);
        self.file_load.loading = false;

        if let Some(name) = first_cell {
            self.select_cell(&name);
        }
    }

    /// Switches to a new cell, cancelling any in-flight element streaming and starting
    /// a new streaming thread for the selected cell's elements.
    fn select_cell(&mut self, name: &str) {
        if let Some(cell) = self.cell.as_mut() {
            cell.selected_cell = Some(name.to_string());
            cell.element_receiver = None;
            cell.elements.clear();
            cell.layers.clear();
            cell.spatial_grid = None;

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

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        ui.close_kind(egui::UiKind::Menu);
                        self.open_file_dialog();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
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
                    if let Some((wx, wy)) = self.mouse_world_pos {
                        ui.label(format!("({wx:.6}, {wy:.6})"));
                    }
                });
            });
        });

        let mut cell_changed = false;
        let mut zoom_to_fit = false;
        let cell = &mut self.cell;
        let layer_state = &mut self.layer_state;
        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                if let Some(cell) = cell.as_mut() {
                    zoom_to_fit = panels::draw_side_panel(
                        ui,
                        &cell.cell_names,
                        &mut cell.selected_cell,
                        &mut cell_changed,
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

        if zoom_to_fit {
            self.zoom_to_fit();
        }

        let cell = &mut self.cell;
        let viewport = &mut self.viewport;
        let layer_state = &mut self.layer_state;
        let mouse_world_pos = &mut self.mouse_world_pos;
        let render_cache = &mut self.render_cache;
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
            );
        });
    }
}
