use pyo3::prelude::*;

use crate::utils::{
    io::{create_temp_file, from_gds, write_gds},
    transformations::{py_any_path_to_string, py_any_path_to_string_or_temp_name},
};

use super::Library;

#[pymethods]
impl Library {
    #[pyo3(signature=(file_name=None, units=1e-6, precision=1e-10))]
    pub fn to_gds(
        &self,
        #[pyo3(from_py_with = "py_any_path_to_string_or_temp_name")] file_name: Option<String>,
        units: f64,
        precision: f64,
    ) -> PyResult<String> {
        write_gds(
            file_name.unwrap_or(create_temp_file()?),
            &self.name,
            units,
            precision,
            self.cells.clone(),
        )
    }

    #[staticmethod]
    pub fn from_gds(
        #[pyo3(from_py_with = "py_any_path_to_string")] file_name: String,
    ) -> PyResult<Library> {
        from_gds(file_name)
    }
}
