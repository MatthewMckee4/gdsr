use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use gdsr::{Element, Library};

use crate::colors::LayerColorMap;
use crate::drawable::Drawable;
use crate::panels;
use crate::spatial::SpatialGrid;
use crate::viewport::{self, Viewport};

/// Tracks an in-flight file-open operation.
#[derive(Default)]
struct FileLoadState {
    file_path: Option<PathBuf>,
    load_receiver: Option<(PathBuf, mpsc::Receiver<Result<Library, String>>)>,
    loading: bool,
    error_message: Option<String>,
}

/// Holds the loaded library, selected cell, and its streamed elements.
#[derive(Default)]
struct CellState {
    library: Option<Library>,
    cell_names: Vec<String>,
    selected_cell: Option<String>,
    elements: Vec<Element>,
    element_receiver: Option<mpsc::Receiver<Element>>,
    elements_loading: bool,
    layers: BTreeSet<(u16, u16)>,
    spatial_grid: Option<SpatialGrid>,
}

#[derive(Default)]
pub struct ViewerApp {
    file_load: FileLoadState,
    cell: CellState,
    viewport: Viewport,
    layer_colors: LayerColorMap,
    hidden_layers: HashSet<(u16, u16)>,
    mouse_world_pos: Option<(f64, f64)>,
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
    fn on_library_loaded(&mut self, library: Library, path: PathBuf) {
        self.cell.cell_names = {
            let mut names: Vec<String> = library.cells().keys().cloned().collect();
            names.sort();
            names
        };

        let first_cell = self.cell.cell_names.first().cloned();
        self.cell.library = Some(library);
        self.file_load.file_path = Some(path);
        self.file_load.loading = false;

        if let Some(name) = first_cell {
            self.select_cell(&name);
        }
    }

    /// Switches to a new cell, cancelling any in-flight element streaming and starting
    /// a new streaming thread for the selected cell's elements.
    fn select_cell(&mut self, name: &str) {
        self.cell.selected_cell = Some(name.to_string());

        // Drop old receiver to cancel any in-flight streaming thread
        self.cell.element_receiver = None;
        self.cell.elements.clear();
        self.cell.layers.clear();
        self.cell.spatial_grid = None;

        if let Some(library) = &self.cell.library {
            if let Some(cell) = library.get_cell(name) {
                let cell = cell.clone();
                let library = library.clone();
                let (tx, rx) = mpsc::channel();

                thread::spawn(move || {
                    cell.stream_elements(None, &library, &tx);
                });

                self.cell.element_receiver = Some(rx);
                self.cell.elements_loading = true;
            }
        }
    }

    /// Adjusts the viewport to fit all currently loaded elements.
    fn zoom_to_fit(&mut self) {
        if let Some(bounds) = viewport::compute_bounds(&self.cell.elements) {
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(800.0, 600.0));
            self.viewport.zoom_to_fit(&bounds, rect);
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
        if let Some(rx) = &self.cell.element_receiver {
            loop {
                match rx.try_recv() {
                    Ok(element) => {
                        for key in element.layer_keys() {
                            if self.cell.layers.insert(key) {
                                self.layer_colors.get(key.0, key.1);
                            }
                        }
                        self.cell.elements.push(element);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.cell.elements_loading = false;
                        self.cell.element_receiver = None;
                        if let Some(bounds) = viewport::compute_bounds(&self.cell.elements) {
                            self.cell.spatial_grid =
                                Some(SpatialGrid::build(&self.cell.elements, &bounds));
                        }
                        self.zoom_to_fit();
                        break;
                    }
                }
            }
        }

        if self.cell.elements_loading {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open...").clicked() {
                        ui.close_menu();
                        self.open_file_dialog();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.file_load.loading {
                    ui.label("Loading...");
                } else if self.cell.elements_loading {
                    ui.label(format!(
                        "Expanding elements... ({})",
                        self.cell.elements.len()
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
        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                zoom_to_fit = panels::draw_side_panel(
                    ui,
                    &self.cell.cell_names,
                    &mut self.cell.selected_cell,
                    &mut cell_changed,
                    &self.cell.layers,
                    &mut self.hidden_layers,
                    &mut self.layer_colors,
                );
            });

        if cell_changed {
            if let Some(name) = self.cell.selected_cell.clone() {
                self.select_cell(&name);
            }
        }

        if zoom_to_fit {
            self.zoom_to_fit();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.mouse_world_pos = viewport::draw_viewport(
                ui,
                &mut self.viewport,
                &self.cell.elements,
                &self.hidden_layers,
                &mut self.layer_colors,
                self.cell.spatial_grid.as_ref(),
            );
        });
    }
}
