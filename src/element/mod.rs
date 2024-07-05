use pyo3::prelude::*;

pub trait Element: IntoPy<PyObject> {
    fn __str__(&self) -> PyResult<String>;
}
