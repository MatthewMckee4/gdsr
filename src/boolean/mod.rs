use geo::BooleanOps;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::element::Element;
use crate::polygon::Polygon;
use crate::traits::FromGeo;
use crate::traits::ToGeo;
use geo::MultiPolygon;

fn check_for_text(elements: &Vec<Element>) -> PyResult<()> {
    for element in elements {
        if let Element::Text(_) = element {
            return Err(PyValueError::new_err(
                "Text elements are not allowed in boolean operations",
            ));
        }
    }
    Ok(())
}

fn get_geo_multi_polygon(elements: &[Element]) -> PyResult<MultiPolygon> {
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

#[pyfunction]
#[pyo3(signature = (a, b, operation, layer=0, data_type=0))]
pub fn boolean(
    a: Vec<Element>,
    b: Vec<Element>,
    operation: String,
    layer: i32,
    data_type: i32,
) -> PyResult<Vec<Polygon>> {
    check_for_text(&a)?;
    check_for_text(&b)?;

    let geo_a = get_geo_multi_polygon(&a)?;
    let geo_b = get_geo_multi_polygon(&b)?;

    let result = match operation.as_str() {
        "or" => geo_a.union(&geo_b),
        "and" => geo_a.intersection(&geo_b),
        "sub" => geo_a.difference(&geo_b),
        "xor" => geo_a.xor(&geo_b),
        _ => return Err(PyValueError::new_err("Invalid operation")),
    };

    Ok(Polygon::from_geo(result, layer, data_type))
}
