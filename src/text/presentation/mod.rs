use pyo3::{exceptions::PyValueError, prelude::*};

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalPresentation {
    Top = 0,
    Middle = 1,
    Bottom = 2,
}

#[pymethods]
impl VerticalPresentation {
    #[new]
    fn new(value: i32) -> PyResult<Self> {
        match value {
            0 => Ok(VerticalPresentation::Top),
            1 => Ok(VerticalPresentation::Middle),
            2 => Ok(VerticalPresentation::Bottom),
            _ => Err(PyValueError::new_err(
                "Invalid value for VerticalPresentation",
            )),
        }
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        match self {
            VerticalPresentation::Top => Ok("Top".to_string()),
            VerticalPresentation::Middle => Ok("Middle".to_string()),
            VerticalPresentation::Bottom => Ok("Bottom".to_string()),
        }
    }

    #[getter]
    fn value(&self) -> PyResult<i32> {
        match self {
            VerticalPresentation::Top => Ok(0),
            VerticalPresentation::Middle => Ok(1),
            VerticalPresentation::Bottom => Ok(2),
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalPresentation {
    Left = 0,
    Centre = 1,
    Right = 2,
}

#[pymethods]
impl HorizontalPresentation {
    #[new]
    fn new(value: i32) -> PyResult<Self> {
        match value {
            0 => Ok(HorizontalPresentation::Left),
            1 => Ok(HorizontalPresentation::Centre),
            2 => Ok(HorizontalPresentation::Right),
            _ => Err(PyValueError::new_err(
                "Invalid value for HorizontalPresentation",
            )),
        }
    }

    #[getter]
    fn name(&self) -> PyResult<String> {
        match self {
            HorizontalPresentation::Left => Ok("Left".to_string()),
            HorizontalPresentation::Centre => Ok("Centre".to_string()),
            HorizontalPresentation::Right => Ok("Right".to_string()),
        }
    }

    #[getter]
    fn value(&self) -> PyResult<i32> {
        match self {
            HorizontalPresentation::Left => Ok(0),
            HorizontalPresentation::Centre => Ok(1),
            HorizontalPresentation::Right => Ok(2),
        }
    }
}
