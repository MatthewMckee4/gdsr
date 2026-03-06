use std::collections::BTreeSet;

use egui::{Color32, Pos2, Stroke, Ui};
use gdsr::{CellStats, DataType, Layer};

use crate::hierarchy::{CellTreeNode, ExpandState};
use crate::state::{CellViewMode, LayerState, SidePanelTab};

const INDENT_PX: f32 = 16.0;
const GUIDE_COLOR: Color32 = Color32::from_gray(60);
const EXPANDED_COLOR: Color32 = Color32::from_gray(140);

/// Draws the side panel content, dispatching to cell or layer panel based on active tab.
pub fn draw_side_panel(
    ui: &mut Ui,
    active_tab: SidePanelTab,
    cell_tree: &[CellTreeNode],
    flat_tree: &[CellTreeNode],
    view_mode: CellViewMode,
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    color_changed: &mut bool,
    expand_state: &mut ExpandState,
    scroll_to_selected: &mut bool,
    layers: &BTreeSet<(Layer, DataType)>,
    layer_state: &mut LayerState,
) {
    match active_tab {
        SidePanelTab::Cells => {
            let tree = match view_mode {
                CellViewMode::Tree => cell_tree,
                CellViewMode::Flat => flat_tree,
            };
            draw_cell_panel(
                ui,
                tree,
                selected_cell,
                cell_changed,
                expand_state,
                scroll_to_selected,
            );
        }
        SidePanelTab::Layers => {
            draw_layer_panel(ui, layers, layer_state, color_changed);
        }
    }
}

fn draw_cell_panel(
    ui: &mut Ui,
    cell_tree: &[CellTreeNode],
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    expand_state: &mut ExpandState,
    scroll_to_selected: &mut bool,
) {
    egui::ScrollArea::vertical()
        .id_salt("cell_tree")
        .show(ui, |ui| {
            for node in cell_tree {
                draw_tree_node(
                    ui,
                    node,
                    selected_cell,
                    cell_changed,
                    expand_state,
                    0,
                    scroll_to_selected,
                );
            }
        });
}

fn draw_layer_panel(
    ui: &mut Ui,
    layers: &BTreeSet<(Layer, DataType)>,
    layer_state: &mut LayerState,
    color_changed: &mut bool,
) {
    egui::ScrollArea::vertical()
        .id_salt("layers")
        .show(ui, |ui| {
            for &(layer, dt) in layers {
                let mut color = layer_state.layer_colors.get(layer, dt);
                let visible = !layer_state.hidden_layers.contains(&(layer, dt));

                ui.horizontal(|ui| {
                    let response = ui.color_edit_button_srgba(&mut color);
                    if response.changed() {
                        layer_state.layer_colors.set(layer, dt, color);
                        *color_changed = true;
                    }

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

/// Draws the statistics detail panel in the bottom bar.
pub fn draw_stats_bar(ui: &mut Ui, stats: &CellStats) {
    ui.separator();

    let parts: Vec<String> = [
        ("P", stats.polygon_count),
        ("Pa", stats.path_count),
        ("B", stats.box_count),
        ("T", stats.text_count),
        ("R", stats.reference_count),
    ]
    .iter()
    .filter(|(_, c)| *c > 0)
    .map(|(label, count)| format!("{label}:{count}"))
    .collect();

    let summary = format!("{} el  {}", stats.total_elements(), parts.join(" "));
    ui.label(summary);
}

fn draw_tree_node(
    ui: &mut Ui,
    node: &CellTreeNode,
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    expand_state: &mut ExpandState,
    depth: usize,
    scroll_to_selected: &mut bool,
) {
    let has_children = !node.children.is_empty();
    let is_selected = selected_cell.as_deref() == Some(&node.name);
    let is_expanded = has_children && expand_state.is_expanded(&node.name);
    let indent = depth as f32 * INDENT_PX;

    let max_width = ui.available_width();
    ui.horizontal(|ui| {
        ui.set_max_width(max_width);

        let base_x = ui.cursor().left();
        let top_y = ui.cursor().top();
        let row_height = ui.spacing().interact_size.y;
        let painter = ui.painter();

        for level in 0..depth {
            let x = base_x + level as f32 * INDENT_PX + INDENT_PX * 0.5;
            painter.line_segment(
                [Pos2::new(x, top_y), Pos2::new(x, top_y + row_height)],
                Stroke::new(1.0, GUIDE_COLOR),
            );
        }

        ui.add_space(indent);

        let label = if is_expanded {
            egui::RichText::new(&node.name).color(EXPANDED_COLOR)
        } else {
            egui::RichText::new(&node.name)
        };

        let response = ui.add(
            egui::Button::selectable(is_selected, label)
                .truncate()
                .frame(false),
        );
        if is_selected && *scroll_to_selected {
            response.scroll_to_me(Some(egui::Align::Center));
            *scroll_to_selected = false;
        }
        if response.clicked() {
            if has_children && is_expanded {
                expand_state.set_expanded(&node.name, false);
            }
            if !is_selected {
                *selected_cell = Some(node.name.clone());
                *cell_changed = true;
                if has_children {
                    expand_state.set_expanded(&node.name, true);
                }
            }
        }
    });

    if is_expanded {
        for child in &node.children {
            draw_tree_node(
                ui,
                child,
                selected_cell,
                cell_changed,
                expand_state,
                depth + 1,
                scroll_to_selected,
            );
        }
    }
}
