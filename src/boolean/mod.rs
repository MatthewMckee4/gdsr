use clipper2::*;
use log::info;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use utils::get_external_polygon_group;

use crate::element::Element;
use crate::polygon::Polygon;
use crate::traits::FromExternalPolygonGroup;

mod utils;

pub type BooleanOperationInput = Vec<Element>;
pub type BooleanOperationOperation = String;
pub type BooleanOperationResult = PyResult<Vec<Polygon>>;

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct CustomPointScaler;

impl PointScaler for CustomPointScaler {
    const MULTIPLIER: f64 = 10000000.0;
}

pub type ExternalPolygonGroup = Paths<CustomPointScaler>;

#[pyfunction]
#[pyo3(signature = (a, b, operation, layer=0, data_type=0))]
pub fn boolean(
    a: BooleanOperationInput,
    b: BooleanOperationInput,
    operation: BooleanOperationOperation,
    layer: i32,
    data_type: i32,
) -> BooleanOperationResult {
    let epg_a = get_external_polygon_group(&a)?;
    let epg_b = get_external_polygon_group(&b)?;

    info!("epg_a: {:?}", epg_a);
    info!("epg_b: {:?}", epg_b);

    let fill_rule = FillRule::EvenOdd;

    let clipper_obj = Clipper::new().add_subject(epg_a).add_clip(epg_b);

    let result = match operation.as_str() {
        "or" => Ok(clipper_obj.union(fill_rule)),
        "and" => Ok(clipper_obj.intersect(fill_rule)),
        "sub" => Ok(clipper_obj.difference(fill_rule)),
        "xor" => Ok(clipper_obj.xor(fill_rule)),
        _ => Err(PyValueError::new_err("Invalid operation")),
    };

    match result {
        Ok(Ok(mp)) => Ok(Polygon::from_external_polygon_group(mp, layer, data_type)?),
        Ok(Err(_)) => Err(PyValueError::new_err("Failed to run boolean operation")),
        Err(e) => Err(e),
    }
}
