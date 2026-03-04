use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use gdsr::{Element, Library};

use crate::colors::LayerColorMap;
use crate::panels;
use crate::spatial::SpatialGrid;
use crate::viewport::{self, Viewport};

#[derive(Default)]
pub struct ViewerApp {
    library: Option<Library>,
    file_path: Option<PathBuf>,
    selected_cell: Option<String>,
    cell_names: Vec<String>,
    elements: Vec<Element>,
    element_receiver: Option<mpsc::Receiver<Element>>,
    elements_loading: bool,
    layers: BTreeSet<(u16, u16)>,
    viewport: Viewport,
    layer_colors: LayerColorMap,
    hidden_layers: HashSet<(u16, u16)>,
    load_receiver: Option<(PathBuf, mpsc::Receiver<Result<Library, String>>)>,
    spatial_grid: Option<SpatialGrid>,
    mouse_world_pos: Option<(f64, f64)>,
    error_message: Option<String>,
    loading: bool,
}

impl ViewerApp {
    fn open_file_dialog(&mut self) {
        if let Some((path, rx)) = crate::loader::load_file_dialog() {
            self.load_receiver = Some((path, rx));
            self.loading = true;
            self.error_message = None;
        }
    }

    fn on_library_loaded(&mut self, library: Library, path: PathBuf) {
        self.cell_names = {
            let mut names: Vec<String> = library.cells().keys().cloned().collect();
            names.sort();
            names
        };

        let first_cell = self.cell_names.first().cloned();
        self.library = Some(library);
        self.file_path = Some(path);
        self.loading = false;

        if let Some(name) = first_cell {
            self.select_cell(&name);
        }
    }

    fn select_cell(&mut self, name: &str) {
        self.selected_cell = Some(name.to_string());

        // Drop old receiver to cancel any in-flight streaming thread
        self.element_receiver = None;
        self.elements.clear();
        self.layers.clear();
        self.spatial_grid = None;

        if let Some(library) = &self.library {
            if let Some(cell) = library.get_cell(name) {
                let cell = cell.clone();
                let library = library.clone();
                let (tx, rx) = mpsc::channel();

                thread::spawn(move || {
                    cell.stream_elements(None, &library, &tx);
                });

                self.element_receiver = Some(rx);
                self.elements_loading = true;
            }
        }
    }

    fn zoom_to_fit(&mut self) {
        if let Some((min_x, min_y, max_x, max_y)) = viewport::compute_bounds(&self.elements) {
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(800.0, 600.0));
            self.viewport.zoom_to_fit(min_x, min_y, max_x, max_y, rect);
        }
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background loader
        if let Some((path, rx)) = self.load_receiver.take() {
            match rx.try_recv() {
                Ok(Ok(library)) => {
                    self.on_library_loaded(library, path);
                }
                Ok(Err(err)) => {
                    self.error_message = Some(err);
                    self.loading = false;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // Still loading, put receiver back
                    self.load_receiver = Some((path, rx));
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.error_message = Some("File loading thread disconnected".to_string());
                    self.loading = false;
                }
            }
        }

        // Drain element streaming channel
        if let Some(rx) = &self.element_receiver {
            loop {
                match rx.try_recv() {
                    Ok(element) => {
                        match &element {
                            Element::Polygon(p) => {
                                let key = (p.layer(), p.data_type());
                                if self.layers.insert(key) {
                                    self.layer_colors.get(key.0, key.1);
                                }
                            }
                            Element::Path(p) => {
                                let key = (p.layer(), p.data_type());
                                if self.layers.insert(key) {
                                    self.layer_colors.get(key.0, key.1);
                                }
                            }
                            Element::Text(t) => {
                                let key = (t.layer(), 0);
                                if self.layers.insert(key) {
                                    self.layer_colors.get(key.0, key.1);
                                }
                            }
                            Element::Reference(_) => {}
                        }
                        self.elements.push(element);
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.elements_loading = false;
                        self.element_receiver = None;
                        if let Some(bounds) = viewport::compute_bounds(&self.elements) {
                            self.spatial_grid = Some(SpatialGrid::build(&self.elements, bounds));
                        }
                        self.zoom_to_fit();
                        break;
                    }
                }
            }
        }

        if self.elements_loading {
            ctx.request_repaint();
        }

        // Top menu bar
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

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.loading {
                    ui.label("Loading...");
                } else if self.elements_loading {
                    ui.label(format!("Expanding elements... ({})", self.elements.len()));
                } else if let Some(err) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
                } else if let Some(path) = &self.file_path {
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

        // Left side panel
        let mut cell_changed = false;
        let mut zoom_to_fit = false;
        egui::SidePanel::left("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                zoom_to_fit = panels::draw_side_panel(
                    ui,
                    &self.cell_names,
                    &mut self.selected_cell,
                    &mut cell_changed,
                    &self.layers,
                    &mut self.hidden_layers,
                    &mut self.layer_colors,
                );
            });

        // Handle cell change after panel is done (to avoid borrow conflicts)
        if cell_changed {
            if let Some(name) = self.selected_cell.clone() {
                self.select_cell(&name);
            }
        }

        if zoom_to_fit {
            self.zoom_to_fit();
        }

        // Central viewport
        egui::CentralPanel::default().show(ctx, |ui| {
            self.mouse_world_pos = viewport::draw_viewport(
                ui,
                &mut self.viewport,
                &self.elements,
                &self.hidden_layers,
                &mut self.layer_colors,
                self.spatial_grid.as_ref(),
            );
        });
    }
}
