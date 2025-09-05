use super::presentation::{HorizontalPresentation, VerticalPresentation};

pub const fn get_presentation_value(
    vertical_presentation: VerticalPresentation,
    horizontal_presentation: HorizontalPresentation,
) -> u16 {
    let vertical_value = vertical_presentation.value();
    let horizontal_value = horizontal_presentation.value();

    match (vertical_value, horizontal_value) {
        (0, 1) => 1,
        (0, 2) => 2,
        (1, 0) => 4,
        (1, 1) => 5,
        (1, 2) => 6,
        (2, 0) => 8,
        (2, 1) => 9,
        (2, 2) => 10,
        _ => 0,
    }
}

pub fn get_presentations_from_value(
    value: i16,
) -> Result<(VerticalPresentation, HorizontalPresentation), String> {
    let (vertical_value, horizontal_value) = match value {
        0 => (0, 0),
        1 => (0, 1),
        2 => (0, 2),
        4 => (1, 0),
        5 => (1, 1),
        6 => (1, 2),
        8 => (2, 0),
        9 => (2, 1),
        10 => (2, 2),
        _ => return Err("Invalid presentation value".to_string()),
    };

    let vertical_presentation = VerticalPresentation::new(vertical_value)?;
    let horizontal_presentation = HorizontalPresentation::new(horizontal_value)?;

    Ok((vertical_presentation, horizontal_presentation))
}
