use pyo3::PyResult;

use super::presentation;

pub fn get_presentation_value(
    vertical_presentation: presentation::VerticalPresentation,
    horizontal_presentation: presentation::HorizontalPresentation,
) -> PyResult<u16> {
    let vertical_value = vertical_presentation.value()?;
    let horizontal_value = horizontal_presentation.value()?;

    let combined_value = match (vertical_value, horizontal_value) {
        (0, 0) => 0,  // NW
        (0, 1) => 1,  // N
        (0, 2) => 2,  // NE
        (1, 0) => 4,  // W
        (1, 1) => 5,  // O
        (1, 2) => 6,  // E
        (2, 0) => 8,  // SW
        (2, 1) => 9,  // S
        (2, 2) => 10, // SE
        _ => Err(pyo3::exceptions::PyValueError::new_err(
            "Invalid combination of presentations",
        ))?,
    };

    Ok(combined_value)
}
