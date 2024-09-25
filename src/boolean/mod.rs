use geo::{BooleanOps, MultiPolygon};
use log::error;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use utils::get_external_polygon_group;

use crate::element::Element;
use crate::polygon::Polygon;
use crate::traits::FromGeo;

mod utils;

pub type BooleanOperationInput = Vec<Element>;
pub type BooleanOperationOperation = String;
pub type BooleanOperationResult = PyResult<Vec<Polygon>>;

pub type ExternalPolygonGroup = MultiPolygon<f64>;

#[pyfunction]
#[pyo3(signature = (a, b, operation, layer=0, data_type=0))]
pub fn boolean(
    a: BooleanOperationInput,
    b: BooleanOperationInput,
    operation: BooleanOperationOperation,
    layer: i32,
    data_type: i32,
) -> BooleanOperationResult {
    let geo_a = get_external_polygon_group(&a)?;
    let geo_b = get_external_polygon_group(&b)?;

    let result = std::panic::catch_unwind(|| match operation.as_str() {
        "or" => Ok(geo_a.union(&geo_b)),
        "and" => Ok(geo_a.intersection(&geo_b)),
        "sub" => Ok(geo_a.difference(&geo_b)),
        "xor" => Ok(geo_a.xor(&geo_b)),
        _ => Err(PyValueError::new_err("Invalid operation")),
    });

    match result {
        Ok(Ok(mp)) => Ok(Polygon::from_geo(mp, layer, data_type)?),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Panic occurred during the operation");
            Ok(vec![])
        }
    }
}
