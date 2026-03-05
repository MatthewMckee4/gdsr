use std::collections::HashMap;

use egui::Color32;
use gdsr::{DataType, Layer};

const PALETTE: [Color32; 16] = [
    Color32::from_rgb(230, 25, 75),
    Color32::from_rgb(60, 180, 75),
    Color32::from_rgb(255, 225, 25),
    Color32::from_rgb(0, 130, 200),
    Color32::from_rgb(245, 130, 48),
    Color32::from_rgb(145, 30, 180),
    Color32::from_rgb(70, 240, 240),
    Color32::from_rgb(240, 50, 230),
    Color32::from_rgb(210, 245, 60),
    Color32::from_rgb(250, 190, 212),
    Color32::from_rgb(0, 128, 128),
    Color32::from_rgb(220, 190, 255),
    Color32::from_rgb(170, 110, 40),
    Color32::from_rgb(255, 250, 200),
    Color32::from_rgb(128, 0, 0),
    Color32::from_rgb(128, 128, 0),
];

/// Maps (layer, datatype) pairs to distinct colors from a fixed palette.
#[derive(Default)]
pub struct LayerColorMap {
    map: HashMap<(Layer, DataType), Color32>,
    next_index: usize,
}

impl LayerColorMap {
    /// Returns the color for the given layer/datatype pair, assigning one if needed.
    pub fn get(&mut self, layer: Layer, datatype: DataType) -> Color32 {
        *self.map.entry((layer, datatype)).or_insert_with(|| {
            let color = PALETTE[self.next_index % PALETTE.len()];
            self.next_index += 1;
            color
        })
    }

    /// Sets the color for the given layer/datatype pair.
    pub fn set(&mut self, layer: Layer, datatype: DataType, color: Color32) {
        self.map.insert((layer, datatype), color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_layer_returns_same_color() {
        let mut map = LayerColorMap::default();
        let c1 = map.get(Layer::new(1), DataType::new(0));
        let c2 = map.get(Layer::new(1), DataType::new(0));
        assert_eq!(c1, c2);
    }

    #[test]
    fn different_layers_get_different_colors() {
        let mut map = LayerColorMap::default();
        let c1 = map.get(Layer::new(1), DataType::new(0));
        let c2 = map.get(Layer::new(2), DataType::new(0));
        assert_ne!(c1, c2);
    }

    #[test]
    fn different_datatypes_get_different_colors() {
        let mut map = LayerColorMap::default();
        let c1 = map.get(Layer::new(1), DataType::new(0));
        let c2 = map.get(Layer::new(1), DataType::new(1));
        assert_ne!(c1, c2);
    }

    #[test]
    fn colors_wrap_around_palette() {
        let mut map = LayerColorMap::default();
        let first = map.get(Layer::new(0), DataType::new(0));
        for i in 1..PALETTE.len() as u16 {
            map.get(Layer::new(i), DataType::new(0));
        }
        let wrapped = map.get(Layer::new(PALETTE.len() as u16), DataType::new(0));
        assert_eq!(first, wrapped);
    }
}
