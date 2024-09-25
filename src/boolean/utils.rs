use crate::{element::Element, traits::ToGeo};

use pyo3::prelude::*;

use super::ExternalPolygonGroup;

pub fn get_external_polygon_group(elements: &[Element]) -> PyResult<ExternalPolygonGroup> {
    let geo_a = ExternalPolygonGroup::new(
        elements
            .iter()
            .filter_map(|e| {
                if let Ok(multi_polygon) = e.to_geo() {
                    Some(multi_polygon)
                } else {
                    None
                }
            })
            .flatten()
            .collect(),
    );

    Ok(geo_a)
}
