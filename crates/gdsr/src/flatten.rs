use std::collections::HashSet;

use crate::{Element, Layer};

/// Controls recursive cell flattening.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FlattenOptions {
    /// Maximum reference depth to expand, or `None` to expand fully.
    pub depth: Option<usize>,
    /// Cell names eligible for expansion, or `None` to expand every referenced cell.
    ///
    /// References to cells outside this set are preserved. An empty set preserves every
    /// named-cell reference.
    pub cells: Option<HashSet<String>>,
    /// Layers to include in the concrete output, or `None` to include every layer.
    ///
    /// Preserved references are not affected because they do not have a layer.
    pub layers: Option<HashSet<Layer>>,
}

impl FlattenOptions {
    pub(crate) fn includes_cell(&self, name: &str) -> bool {
        self.cells.as_ref().is_none_or(|cells| cells.contains(name))
    }

    pub(crate) fn includes_element(&self, element: &Element) -> bool {
        let layer = match element {
            Element::Path(path) => path.layer(),
            Element::Polygon(polygon) => polygon.layer(),
            Element::Box(gds_box) => gds_box.layer(),
            Element::Node(node) => node.layer(),
            Element::Text(text) => text.layer(),
            Element::Reference(_) => return true,
        };

        self.layers
            .as_ref()
            .is_none_or(|layers| layers.contains(&layer))
    }
}
