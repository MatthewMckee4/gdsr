use pyo3::{exceptions::PyValueError, prelude::*};

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PathType {
    Square = 0,
    Round = 1,
    Overlap = 2,
}

#[pymethods]
impl PathType {
    #[new]
    fn new(value: i32) -> PyResult<Self> {
        match value {
            0 => Ok(PathType::Square),
            1 => Ok(PathType::Round),
            2 => Ok(PathType::Overlap),
            _ => Err(PyValueError::new_err("Invalid value for PathType")),
        }
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        match self {
            PathType::Square => Ok("Square Ends".to_string()),
            PathType::Round => Ok("Round Ends".to_string()),
            PathType::Overlap => Ok("Overlap Ends".to_string()),
        }
    }

    #[getter]
    pub fn value(&self) -> PyResult<i32> {
        Ok(*self as i32)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self))
    }
}

impl std::fmt::Display for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name())
    }
}

impl std::fmt::Debug for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name())
    }
}
