use crate::{element::Element, traits::ToGeo};
use geo::MultiPolygon;
use pyo3::{exceptions::PyValueError, prelude::*};

pub fn check_for_text(elements: &Vec<Element>) -> PyResult<()> {
    for element in elements {
        if let Element::Text(_) = element {
            return Err(PyValueError::new_err(
                "Text elements are not allowed in boolean operations",
            ));
        }
    }
    Ok(())
}

pub fn get_geo_multi_polygon(elements: &[Element]) -> PyResult<MultiPolygon> {
    let geo_a = MultiPolygon::new(
        elements
            .iter()
            .filter_map(|e| {
                if let Ok(MultiPolygon(multi_polygon)) = e.to_geo() {
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
