use pyo3::prelude::*;

use crate::utils::io::write_gds;

use super::Library;

#[pymethods]
impl Library {
    #[pyo3(signature=(file_name, units=1e-6, precision=1e-10))]
    pub fn to_gds(&self, file_name: &str, units: f64, precision: f64) -> PyResult<()> {
        write_gds(file_name, &self.name, units, precision, self.cells.clone())
    }
}
