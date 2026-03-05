use std::collections::BTreeSet;

use egui::{Ui, Vec2};
use gdsr::{CellStats, DataType, Layer};

use crate::hierarchy::{CellTreeNode, ExpandState};
use crate::state::LayerState;

/// Draws the side panel with cell hierarchy tree, statistics, and layer toggles.
pub fn draw_side_panel(
    ui: &mut Ui,
    cell_tree: &[CellTreeNode],
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    expand_state: &mut ExpandState,
    layers: &BTreeSet<(Layer, DataType)>,
    layer_state: &mut LayerState,
    cell_stats: Option<&CellStats>,
) {
    ui.heading("Cells");

    ui.horizontal(|ui| {
        if ui.small_button("Expand All").clicked() {
            expand_state.expand_all(cell_tree);
        }
        if ui.small_button("Collapse All").clicked() {
            expand_state.collapse_all(cell_tree);
        }
    });

    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .id_salt("cell_tree")
        .max_height(ui.available_height() * 0.5)
        .show(ui, |ui| {
            for node in cell_tree {
                draw_tree_node(ui, node, selected_cell, cell_changed, expand_state);
            }
        });

    if let Some(stats) = cell_stats {
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);

        ui.collapsing("Statistics", |ui| {
            ui.strong("Elements");
            egui::Grid::new("stats_grid")
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    let rows: &[(&str, usize)] = &[
                        ("Polygons", stats.polygon_count),
                        ("Paths", stats.path_count),
                        ("Boxes", stats.box_count),
                        ("Texts", stats.text_count),
                        ("References", stats.reference_count),
                    ];
                    for &(label, count) in rows {
                        if count > 0 {
                            ui.label(label);
                            ui.label(count.to_string());
                            ui.end_row();
                        }
                    }

                    ui.label("Total");
                    ui.label(stats.total_elements().to_string());
                    ui.end_row();
                });

            ui.separator();

            if !stats.elements_per_layer.is_empty() {
                ui.strong("Elements per layer");
                egui::Grid::new("layer_stats_grid")
                    .num_columns(2)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        for (&(layer, dt), &count) in &stats.elements_per_layer {
                            ui.label(format!("L{layer} D{dt}"));
                            ui.label(count.to_string());
                            ui.end_row();
                        }
                    });
            }

            if !stats.references_per_cell.is_empty() {
                ui.add_space(4.0);
                ui.strong("References per cell");
                egui::Grid::new("ref_stats_grid")
                    .num_columns(2)
                    .spacing([8.0, 2.0])
                    .show(ui, |ui| {
                        for (name, &count) in &stats.references_per_cell {
                            ui.label(name.as_str());
                            ui.label(count.to_string());
                            ui.end_row();
                        }

                        ui.label("Total");
                        ui.label(stats.reference_count.to_string());
                        ui.end_row();
                    });

                ui.separator();
            }
        });
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(4.0);

    ui.heading("Layers");
    egui::ScrollArea::vertical()
        .id_salt("layers")
        .max_height(ui.available_height() - 40.0)
        .show(ui, |ui| {
            for &(layer, dt) in layers {
                let color = layer_state.layer_colors.get(layer, dt);
                let visible = !layer_state.hidden_layers.contains(&(layer, dt));

                ui.horizontal(|ui| {
                    let (response, painter) =
                        ui.allocate_painter(Vec2::new(14.0, 14.0), egui::Sense::hover());
                    painter.rect_filled(response.rect, 2.0, color);

                    let mut checked = visible;
                    if ui
                        .checkbox(&mut checked, format!("L{layer} D{dt}"))
                        .changed()
                    {
                        if checked {
                            layer_state.hidden_layers.remove(&(layer, dt));
                        } else {
                            layer_state.hidden_layers.insert((layer, dt));
                        }
                    }
                });
            }
        });
}

fn draw_tree_node(
    ui: &mut Ui,
    node: &CellTreeNode,
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    expand_state: &mut ExpandState,
) {
    let has_children = !node.children.is_empty();
    let is_selected = selected_cell.as_deref() == Some(&node.name);

    if has_children {
        let open = expand_state.is_expanded(&node.name);
        let id = ui.make_persistent_id(&node.name);
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open);

        if state.is_open() != open {
            state.set_open(open);
        }

        ui.spacing_mut().indent = 10.0;
        state
            .show_header(ui, |ui| {
                if ui.selectable_label(is_selected, &node.name).clicked() && !is_selected {
                    *selected_cell = Some(node.name.clone());
                    *cell_changed = true;
                }
            })
            .body(|ui| {
                for child in &node.children {
                    draw_tree_node(ui, child, selected_cell, cell_changed, expand_state);
                }
            });

        if let Some(persisted) = egui::collapsing_header::CollapsingState::load(ui.ctx(), id) {
            let new_open = persisted.is_open();
            if new_open != open {
                expand_state.set_expanded(&node.name, new_open);
            }
        }
    } else {
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            if ui.selectable_label(is_selected, &node.name).clicked() && !is_selected {
                *selected_cell = Some(node.name.clone());
                *cell_changed = true;
            }
        });
    }
}
