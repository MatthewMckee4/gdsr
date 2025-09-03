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
            0 => Ok(VerticalPresentation::Top),
            1 => Ok(VerticalPresentation::Middle),
            2 => Ok(VerticalPresentation::Bottom),
            _ => Err("Invalid value for VerticalPresentation".to_string()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            VerticalPresentation::Top => "Top",
            VerticalPresentation::Middle => "Middle",
            VerticalPresentation::Bottom => "Bottom",
        }
    }

    pub fn value(&self) -> i32 {
        *self as i32
    }

    pub fn values() -> Vec<VerticalPresentation> {
        vec![
            VerticalPresentation::Top,
            VerticalPresentation::Middle,
            VerticalPresentation::Bottom,
        ]
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
            0 => Ok(HorizontalPresentation::Left),
            1 => Ok(HorizontalPresentation::Centre),
            2 => Ok(HorizontalPresentation::Right),
            _ => Err("Invalid value for HorizontalPresentation".to_string()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            HorizontalPresentation::Left => "Left",
            HorizontalPresentation::Centre => "Centre",
            HorizontalPresentation::Right => "Right",
        }
    }

    pub fn value(&self) -> i32 {
        *self as i32
    }

    pub fn values() -> Vec<HorizontalPresentation> {
        vec![
            HorizontalPresentation::Left,
            HorizontalPresentation::Centre,
            HorizontalPresentation::Right,
        ]
    }
}
