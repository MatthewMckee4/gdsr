use pyo3::prelude::*;

pub mod gds_file_types;

pub const FLOATING_POINT_INACCURACY_ROUND_DECIMALS: u32 = 10;
pub static mut EPSILON: f64 = 1e-4;

#[pyfunction]
pub fn set_epsilon(epsilon: f64) {
    unsafe {
        EPSILON = epsilon;
    }
}

#[pyfunction]
pub fn get_epsilon() -> f64 {
    unsafe { EPSILON }
}

pub fn epsilon_is_close(a: f64, b: f64) -> bool {
    unsafe { (a - b).abs() < EPSILON }
}
