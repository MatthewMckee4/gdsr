use std::collections::BTreeSet;

use egui::{Color32, Pos2, Stroke, Ui};
use gdsr::{CellStats, DataType, Element, Layer, Point};

use crate::hierarchy::{CellTreeNode, ExpandState};
use crate::state::{CellViewMode, LayerState, SidePanelTab};

const INDENT_PX: f32 = 16.0;
const GUIDE_COLOR: Color32 = Color32::from_gray(60);
const EXPANDED_COLOR: Color32 = Color32::from_gray(140);

#[derive(Default)]
pub struct SidePanelActions {
    pub delete_selected: bool,
    pub create_cell: bool,
    pub rename_cell: bool,
}

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
    selected_element: Option<&Element>,
) -> SidePanelActions {
    let mut actions = SidePanelActions::default();
    if let Some(element) = selected_element {
        actions.delete_selected = draw_element_panel(ui, element);
        ui.separator();
    }

    match active_tab {
        SidePanelTab::Cells => {
            let tree = match view_mode {
                CellViewMode::Tree => cell_tree,
                CellViewMode::Flat => flat_tree,
            };
            let cell_actions = draw_cell_panel(
                ui,
                tree,
                selected_cell,
                cell_changed,
                expand_state,
                scroll_to_selected,
            );
            actions.create_cell = cell_actions.create_cell;
            actions.rename_cell = cell_actions.rename_cell;
        }
        SidePanelTab::Layers => {
            draw_layer_panel(ui, layers, layer_state, color_changed);
        }
    }

    actions
}

fn draw_element_panel(ui: &mut Ui, element: &Element) -> bool {
    let mut delete_selected = false;
    egui::CollapsingHeader::new("Selected element")
        .default_open(true)
        .show(ui, |ui| {
            if ui.button("Delete").clicked() {
                delete_selected = true;
            }
            ui.separator();

            match element {
                Element::Path(path) => {
                    draw_layer_data(ui, "Path", path.layer(), path.data_type());
                    ui.label(format!("Vertices: {}", path.points().len()));
                    ui.label(format!(
                        "Width: {}",
                        path.width()
                            .map_or_else(|| "Unset".to_string(), |width| width.to_string())
                    ));
                    draw_points(ui, path.points());
                }
                Element::Polygon(polygon) => {
                    draw_layer_data(ui, "Polygon", polygon.layer(), polygon.data_type());
                    ui.label(format!(
                        "Vertices: {}",
                        polygon.points().len().saturating_sub(1)
                    ));
                    draw_points(ui, polygon.points());
                }
                Element::Box(gds_box) => {
                    draw_layer_data(ui, "Box", gds_box.layer(), gds_box.box_type());
                    draw_points(ui, &gds_box.points());
                }
                Element::Node(node) => {
                    draw_layer_data(ui, "Node", node.layer(), node.node_type());
                    ui.label(format!("Points: {}", node.points().len()));
                    draw_points(ui, node.points());
                }
                Element::Text(text) => {
                    draw_layer_data(ui, "Text", text.layer(), text.data_type());
                    ui.label(format!("Value: {}", text.text()));
                    ui.label(format!("Origin: {}", format_point(text.origin())));
                }
                Element::Reference(reference) => {
                    ui.label("Type: Reference");
                    ui.label(format!(
                        "Grid origin: {}",
                        format_point(&reference.grid().origin())
                    ));
                }
            }
        });
    delete_selected
}

fn draw_layer_data(ui: &mut Ui, kind: &str, layer: Layer, data_type: DataType) {
    ui.label(format!("Type: {kind}"));
    ui.label(format!("Layer: {layer}"));
    ui.label(format!("Datatype: {data_type}"));
}

fn draw_points(ui: &mut Ui, points: &[Point]) {
    egui::ScrollArea::vertical()
        .id_salt("selected_element_points")
        .max_height(160.0)
        .show(ui, |ui| {
            egui::Grid::new("selected_element_points_grid")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    for (idx, point) in points.iter().enumerate() {
                        ui.label(format!("#{idx}"));
                        ui.monospace(format_point(point));
                        ui.end_row();
                    }
                });
        });
}

fn format_point(point: &Point) -> String {
    format!("({}, {})", point.x(), point.y())
}

fn draw_cell_panel(
    ui: &mut Ui,
    cell_tree: &[CellTreeNode],
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    expand_state: &mut ExpandState,
    scroll_to_selected: &mut bool,
) -> SidePanelActions {
    let mut actions = SidePanelActions::default();
    ui.horizontal(|ui| {
        if ui.button("New Cell").clicked() {
            actions.create_cell = true;
        }
        if ui
            .add_enabled(selected_cell.is_some(), egui::Button::new("Rename"))
            .clicked()
        {
            actions.rename_cell = true;
        }
    });
    ui.separator();

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
    actions
}

fn draw_layer_panel(
    ui: &mut Ui,
    layers: &BTreeSet<(Layer, DataType)>,
    layer_state: &mut LayerState,
    color_changed: &mut bool,
) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        let clear = ui.add_enabled(!layer_state.filter.is_empty(), egui::Button::new("Clear"));
        let filter = ui
            .add(
                egui::TextEdit::singleline(&mut layer_state.filter)
                    .hint_text("Filter layers")
                    .desired_width(f32::INFINITY),
            )
            .on_hover_text("Focus with Command/Ctrl+F");

        if ui.input(|input| input.modifiers.command && input.key_pressed(egui::Key::F)) {
            filter.request_focus();
        }

        if clear.clicked() {
            layer_state.filter.clear();
            filter.request_focus();
        }
    });

    if layer_state
        .selected_layer
        .is_some_and(|selected| !layers.contains(&selected))
    {
        layer_state.selected_layer = None;
        layer_state.exit_solo();
    }

    let filtered_layers: Vec<_> = layer_state.filtered_layers(layers).collect();
    let selected_filtered = layer_state
        .selected_layer
        .filter(|selected| filtered_layers.contains(selected));
    let wants_keyboard_input = ui.ctx().egui_wants_keyboard_input();
    let (show_all_key, hide_all_key, invert_key, toggle_key, solo_key) = ui.input(|input| {
        let enabled = accepts_layer_shortcut(wants_keyboard_input, input.modifiers);
        (
            enabled && input.key_pressed(egui::Key::A),
            enabled && input.key_pressed(egui::Key::H),
            enabled && input.key_pressed(egui::Key::I),
            enabled && input.key_pressed(egui::Key::V),
            enabled && input.key_pressed(egui::Key::S),
        )
    });

    let mut show_all = false;
    let mut hide_all = false;
    let mut invert = false;
    let mut toggle = false;
    let mut solo = false;
    ui.horizontal_wrapped(|ui| {
        show_all = ui
            .add(egui::Button::new("Show All").shortcut_text("A"))
            .on_hover_text("Show filtered layers")
            .clicked();
        hide_all = ui
            .add(egui::Button::new("Hide All").shortcut_text("H"))
            .on_hover_text("Hide filtered layers")
            .clicked();
        invert = ui
            .add(egui::Button::new("Invert").shortcut_text("I"))
            .on_hover_text("Invert filtered layers")
            .clicked();
        toggle = ui
            .add_enabled(
                selected_filtered.is_some(),
                egui::Button::new("Toggle").shortcut_text("V"),
            )
            .on_hover_text("Toggle selected layer")
            .clicked();
        solo = ui
            .add_enabled(
                layer_state.is_solo_active() || selected_filtered.is_some(),
                egui::Button::new(if layer_state.is_solo_active() {
                    "Exit Solo"
                } else {
                    "Solo"
                })
                .shortcut_text("S"),
            )
            .on_hover_text("Show only selected layer, or restore previous visibility")
            .clicked();
    });

    if show_all || show_all_key {
        layer_state.show_layers(&filtered_layers);
    } else if hide_all || hide_all_key {
        layer_state.hide_layers(&filtered_layers);
    } else if invert || invert_key {
        layer_state.invert_layers(&filtered_layers);
    } else if (toggle || toggle_key)
        && let Some(selected) = selected_filtered
    {
        layer_state.toggle_layer(selected);
    } else if solo || solo_key {
        layer_state.toggle_solo(layers, selected_filtered);
    }

    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt("layers")
        .show(ui, |ui| {
            for &(layer, dt) in &filtered_layers {
                let mut color = layer_state.layer_colors.get(layer, dt);
                let visible = !layer_state.hidden_layers.contains(&(layer, dt));

                ui.horizontal(|ui| {
                    let response = ui.color_edit_button_srgba(&mut color);
                    if response.changed() {
                        layer_state.layer_colors.set(layer, dt, color);
                        *color_changed = true;
                    }

                    let mut checked = visible;
                    if ui.checkbox(&mut checked, "").changed() {
                        layer_state.set_layer_visible((layer, dt), checked);
                    }
                    if ui
                        .add(
                            egui::Button::selectable(
                                layer_state.selected_layer == Some((layer, dt)),
                                format!("L{layer} D{dt}"),
                            )
                            .truncate()
                            .frame(false),
                        )
                        .clicked()
                    {
                        layer_state.selected_layer = Some((layer, dt));
                    }
                });
            }
        });
}

fn accepts_layer_shortcut(wants_keyboard_input: bool, modifiers: egui::Modifiers) -> bool {
    !wants_keyboard_input && modifiers.is_none()
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
                Stroke::new(1.0_f32, GUIDE_COLOR),
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

#[cfg(test)]
mod tests {
    use super::accepts_layer_shortcut;

    #[test]
    fn layer_shortcuts_require_unmodified_non_text_input() {
        assert!(accepts_layer_shortcut(false, egui::Modifiers::NONE));
        assert!(!accepts_layer_shortcut(true, egui::Modifiers::NONE));
        assert!(!accepts_layer_shortcut(false, egui::Modifiers::CTRL));
    }
}
