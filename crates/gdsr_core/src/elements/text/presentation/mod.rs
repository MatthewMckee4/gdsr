#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalPresentation {
    Top = 0,
    #[default]
    Middle = 1,
    Bottom = 2,
}

impl std::fmt::Display for VerticalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Vertical {}", self.name())
    }
}

impl std::fmt::Debug for VerticalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl VerticalPresentation {
    pub fn new(value: i32) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Top),
            1 => Ok(Self::Middle),
            2 => Ok(Self::Bottom),
            _ => Err("Invalid value for VerticalPresentation".to_string()),
        }
    }

    #[must_use]
    pub const fn name(&self) -> &str {
        match self {
            Self::Top => "Top",
            Self::Middle => "Middle",
            Self::Bottom => "Bottom",
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub fn values() -> Vec<Self> {
        vec![Self::Top, Self::Middle, Self::Bottom]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum HorizontalPresentation {
    Left = 0,
    #[default]
    Centre = 1,
    Right = 2,
}

impl std::fmt::Display for HorizontalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Horizontal {}", self.name())
    }
}

impl std::fmt::Debug for HorizontalPresentation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl HorizontalPresentation {
    pub fn new(value: i32) -> Result<Self, String> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Centre),
            2 => Ok(Self::Right),
            _ => Err("Invalid value for HorizontalPresentation".to_string()),
        }
    }

    #[must_use]
    pub const fn name(&self) -> &str {
        match self {
            Self::Left => "Left",
            Self::Centre => "Centre",
            Self::Right => "Right",
        }
    }

    #[must_use]
    pub const fn value(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub fn values() -> Vec<Self> {
        vec![Self::Left, Self::Centre, Self::Right]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertical_presentation_new() {
        assert_eq!(VerticalPresentation::new(0), Ok(VerticalPresentation::Top));
        assert_eq!(
            VerticalPresentation::new(1),
            Ok(VerticalPresentation::Middle)
        );
        assert_eq!(
            VerticalPresentation::new(2),
            Ok(VerticalPresentation::Bottom)
        );
        assert!(VerticalPresentation::new(3).is_err());
        assert!(VerticalPresentation::new(-1).is_err());
    }

    #[test]
    fn test_vertical_presentation_name() {
        assert_eq!(VerticalPresentation::Top.name(), "Top");
        assert_eq!(VerticalPresentation::Middle.name(), "Middle");
        assert_eq!(VerticalPresentation::Bottom.name(), "Bottom");
    }

    #[test]
    fn test_vertical_presentation_value() {
        assert_eq!(VerticalPresentation::Top.value(), 0);
        assert_eq!(VerticalPresentation::Middle.value(), 1);
        assert_eq!(VerticalPresentation::Bottom.value(), 2);
    }

    #[test]
    fn test_vertical_presentation_values() {
        let values = VerticalPresentation::values();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&VerticalPresentation::Top));
        assert!(values.contains(&VerticalPresentation::Middle));
        assert!(values.contains(&VerticalPresentation::Bottom));
    }

    #[test]
    fn test_vertical_presentation_default() {
        assert_eq!(
            VerticalPresentation::default(),
            VerticalPresentation::Middle
        );
    }

    #[test]
    fn test_vertical_presentation_display_and_debug() {
        assert_eq!(format!("{}", VerticalPresentation::Top), "Vertical Top");
        assert_eq!(format!("{:?}", VerticalPresentation::Middle), "Middle");
        assert_eq!(
            format!("{}", VerticalPresentation::Bottom),
            "Vertical Bottom"
        );
    }

    #[test]
    fn test_vertical_presentation_clone_copy_eq() {
        let vp = VerticalPresentation::Top;
        let cloned = vp;
        let copied = vp;

        assert_eq!(vp, cloned);
        assert_eq!(vp, copied);
        assert_ne!(vp, VerticalPresentation::Middle);
    }

    #[test]
    fn test_horizontal_presentation_new() {
        assert_eq!(
            HorizontalPresentation::new(0),
            Ok(HorizontalPresentation::Left)
        );
        assert_eq!(
            HorizontalPresentation::new(1),
            Ok(HorizontalPresentation::Centre)
        );
        assert_eq!(
            HorizontalPresentation::new(2),
            Ok(HorizontalPresentation::Right)
        );
        assert!(HorizontalPresentation::new(3).is_err());
        assert!(HorizontalPresentation::new(-1).is_err());
    }

    #[test]
    fn test_horizontal_presentation_name() {
        assert_eq!(HorizontalPresentation::Left.name(), "Left");
        assert_eq!(HorizontalPresentation::Centre.name(), "Centre");
        assert_eq!(HorizontalPresentation::Right.name(), "Right");
    }

    #[test]
    fn test_horizontal_presentation_value() {
        assert_eq!(HorizontalPresentation::Left.value(), 0);
        assert_eq!(HorizontalPresentation::Centre.value(), 1);
        assert_eq!(HorizontalPresentation::Right.value(), 2);
    }

    #[test]
    fn test_horizontal_presentation_values() {
        let values = HorizontalPresentation::values();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&HorizontalPresentation::Left));
        assert!(values.contains(&HorizontalPresentation::Centre));
        assert!(values.contains(&HorizontalPresentation::Right));
    }

    #[test]
    fn test_horizontal_presentation_default() {
        assert_eq!(
            HorizontalPresentation::default(),
            HorizontalPresentation::Centre
        );
    }

    #[test]
    fn test_horizontal_presentation_display_and_debug() {
        assert_eq!(
            format!("{}", HorizontalPresentation::Left),
            "Horizontal Left"
        );
        assert_eq!(format!("{:?}", HorizontalPresentation::Centre), "Centre");
        assert_eq!(
            format!("{}", HorizontalPresentation::Right),
            "Horizontal Right"
        );
    }

    #[test]
    fn test_horizontal_presentation_clone_copy_eq() {
        let hp = HorizontalPresentation::Left;
        let cloned = hp;
        let copied = hp;

        assert_eq!(hp, cloned);
        assert_eq!(hp, copied);
        assert_ne!(hp, HorizontalPresentation::Centre);
    }

    #[test]
    fn test_presentation_error_messages() {
        assert_eq!(
            VerticalPresentation::new(10),
            Err("Invalid value for VerticalPresentation".to_string())
        );
        assert_eq!(
            HorizontalPresentation::new(10),
            Err("Invalid value for HorizontalPresentation".to_string())
        );
    }
}
