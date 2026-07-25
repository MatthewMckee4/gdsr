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

    egui::ScrollArea::vertical()
        .id_salt("layers")
        .show(ui, |ui| {
            for &(layer, dt) in layers
                .iter()
                .filter(|&&(layer, dt)| layer_matches_filter(layer, dt, &layer_state.filter))
            {
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

fn layer_matches_filter(layer: Layer, data_type: DataType, filter: &str) -> bool {
    let filter = filter.trim();
    filter.is_empty()
        || format!("L{layer} D{data_type}")
            .to_ascii_lowercase()
            .contains(&filter.to_ascii_lowercase())
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
    use super::layer_matches_filter;
    use gdsr::{DataType, Layer};

    #[test]
    fn layer_filter_matches_layer_and_datatype() {
        let layer = Layer::new(12);
        let data_type = DataType::new(7);

        assert!(layer_matches_filter(layer, data_type, ""));
        assert!(layer_matches_filter(layer, data_type, "12"));
        assert!(layer_matches_filter(layer, data_type, "D7"));
        assert!(layer_matches_filter(layer, data_type, " l12 d7 "));
        assert!(!layer_matches_filter(layer, data_type, "L7"));
    }
}
