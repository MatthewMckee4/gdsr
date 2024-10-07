use clipper2::*;
use log::info;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use utils::get_external_polygon_group;

use crate::config::get_epsilon;
use crate::element::Element;
use crate::polygon::Polygon;
use crate::traits::FromExternalPolygonGroup;

mod utils;

pub type BooleanOperationInput = Vec<Element>;
pub type BooleanOperationOperation = String;
pub type BooleanOperationResult = PyResult<Vec<Polygon>>;

#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct CustomScale;

impl PointScaler for CustomScale {
    const MULTIPLIER: f64 = 1000000.0;
}

pub type ExternalPolygonGroup = Paths<CustomScale>;

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

    let fill_rule = FillRule::EvenOdd;

    let scale_factor = get_epsilon();

    let result = std::panic::catch_unwind(|| match operation.as_str() {
        "or" => Ok(Clipper::new()
            .add_subject(epg_a)
            .add_clip(epg_b)
            .union(fill_rule)),
        "and" => Ok(Clipper::new()
            .add_subject(epg_a)
            .add_clip(epg_b)
            .intersect(fill_rule)),
        "sub" => Ok(Clipper::new()
            .add_subject(epg_a)
            .add_clip(epg_b)
            .difference(fill_rule)),
        "xor" => Ok(Clipper::new()
            .add_subject(epg_a)
            .add_clip(epg_b)
            .xor(fill_rule)),
        _ => Err(PyValueError::new_err("Invalid operation")),
    });

    match result {
        Ok(Ok(Ok(mp))) => Ok(Polygon::from_external_polygon_group(
            mp.simplify(scale_factor, false),
            layer,
            data_type,
        )?),
        Ok(Ok(Err(_))) => Err(PyValueError::new_err(
            "Failed to run boolean operation".to_string(),
        )),
        Ok(Err(e)) => Err(e),
        Err(e) => {
            info!("Panic occurred during the operation: {:?}", e);
            Err(PyValueError::new_err("Panic occurred during the operation"))
        }
    }
}
