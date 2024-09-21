use geo::BooleanOps;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use utils::{check_for_text, get_geo_multi_polygon};

use crate::element::Element;
use crate::polygon::Polygon;
use crate::traits::FromGeo;

mod utils;

pub type BooleanOperationInput = Vec<Element>;
pub type BooleanOperationOperation = String;
pub type BooleanOperationResult = PyResult<Vec<Polygon>>;

#[pyfunction]
#[pyo3(signature = (a, b, operation, layer=0, data_type=0))]
pub fn boolean(
    a: BooleanOperationInput,
    b: BooleanOperationInput,
    operation: BooleanOperationOperation,
    layer: i32,
    data_type: i32,
) -> BooleanOperationResult {
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
