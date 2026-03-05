use std::collections::BTreeSet;

use egui::{ComboBox, Ui, Vec2};
use gdsr::{DataType, Layer};

use crate::state::LayerState;

/// Draws the side panel with cell selector and layer toggles.
pub fn draw_side_panel(
    ui: &mut Ui,
    cell_names: &[String],
    selected_cell: &mut Option<String>,
    cell_changed: &mut bool,
    layers: &BTreeSet<(Layer, DataType)>,
    layer_state: &mut LayerState,
) {
    ui.heading("Cells");
    let current_label = selected_cell.as_deref().unwrap_or("(none)");

    ComboBox::from_label("")
        .selected_text(current_label)
        .width(ui.available_width() - 16.0)
        .show_ui(ui, |ui| {
            for name in cell_names {
                let is_selected = selected_cell.as_deref() == Some(name.as_str());
                if ui.selectable_label(is_selected, name).clicked() && !is_selected {
                    *selected_cell = Some(name.clone());
                    *cell_changed = true;
                }
            }
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(4.0);

    ui.heading("Layers");
    egui::ScrollArea::vertical()
        .max_height(ui.available_height() - 40.0)
        .show(ui, |ui| {
            for &(layer, dt) in layers {
                let color = layer_state.layer_colors.get(layer, dt);
                let visible = !layer_state.hidden_layers.contains(&(layer, dt));

                ui.horizontal(|ui| {
                    // Color swatch
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
