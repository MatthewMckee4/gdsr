use pyo3::{exceptions::PyValueError, prelude::*};

pub fn check_layer_valid(layer: i32) -> PyResult<()> {
    if !(0..=255).contains(&layer) {
        return Err(PyValueError::new_err("Layer must be in the range 0-255"));
    }
    Ok(())
}

pub fn check_data_type_valid(_: i32) -> PyResult<()> {
    Ok(())
}
