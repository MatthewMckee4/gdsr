use std::collections::BTreeSet;

use egui::{Ui, Vec2};
use gdsr::{DataType, Layer};

use crate::hierarchy::{CellTreeNode, ExpandState};
use crate::state::LayerState;

/// Draws the side panel with cell hierarchy tree and layer toggles.
pub fn draw_side_panel(
    ui: &mut Ui,
    cell_tree: &[CellTreeNode],
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    expand_state: &mut ExpandState,
    layers: &BTreeSet<(Layer, DataType)>,
    layer_state: &mut LayerState,
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
