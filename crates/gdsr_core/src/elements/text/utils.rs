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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_presentation_value_all_combinations() {
        // Test all valid combinations
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(0).unwrap(),
                HorizontalPresentation::new(0).unwrap()
            ),
            0
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(0).unwrap(),
                HorizontalPresentation::new(1).unwrap()
            ),
            1
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(0).unwrap(),
                HorizontalPresentation::new(2).unwrap()
            ),
            2
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(1).unwrap(),
                HorizontalPresentation::new(0).unwrap()
            ),
            4
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(1).unwrap(),
                HorizontalPresentation::new(1).unwrap()
            ),
            5
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(1).unwrap(),
                HorizontalPresentation::new(2).unwrap()
            ),
            6
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(2).unwrap(),
                HorizontalPresentation::new(0).unwrap()
            ),
            8
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(2).unwrap(),
                HorizontalPresentation::new(1).unwrap()
            ),
            9
        );
        assert_eq!(
            get_presentation_value(
                VerticalPresentation::new(2).unwrap(),
                HorizontalPresentation::new(2).unwrap()
            ),
            10
        );
    }

    #[test]
    fn test_get_presentations_from_value_all_valid() {
        let test_cases = vec![
            (0, (0, 0)),
            (1, (0, 1)),
            (2, (0, 2)),
            (4, (1, 0)),
            (5, (1, 1)),
            (6, (1, 2)),
            (8, (2, 0)),
            (9, (2, 1)),
            (10, (2, 2)),
        ];

        for (input, (expected_v, expected_h)) in test_cases {
            let result = get_presentations_from_value(input);
            assert!(result.is_ok(), "Failed for input {input}");
            let (v, h) = result.unwrap();
            assert_eq!(v.value(), expected_v);
            assert_eq!(h.value(), expected_h);
        }
    }

    #[test]
    fn test_get_presentations_from_value_invalid() {
        assert!(get_presentations_from_value(3).is_err());
        assert!(get_presentations_from_value(7).is_err());
        assert!(get_presentations_from_value(11).is_err());
        assert!(get_presentations_from_value(-1).is_err());
        assert!(get_presentations_from_value(100).is_err());
    }

    #[test]
    fn test_roundtrip_presentation_conversion() {
        // Test that converting to value and back gives the same result
        for v_val in 0..=2 {
            for h_val in 0..=2 {
                let v = VerticalPresentation::new(v_val).unwrap();
                let h = HorizontalPresentation::new(h_val).unwrap();
                let value = get_presentation_value(v, h);
                let (v_back, h_back) = get_presentations_from_value(value as i16).unwrap();
                assert_eq!(v, v_back);
                assert_eq!(h, h_back);
            }
        }
    }
}
