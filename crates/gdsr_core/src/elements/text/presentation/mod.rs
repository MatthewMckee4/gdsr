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

    pub const fn name(&self) -> &str {
        match self {
            Self::Top => "Top",
            Self::Middle => "Middle",
            Self::Bottom => "Bottom",
        }
    }

    pub const fn value(self) -> i32 {
        self as i32
    }

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

    pub const fn name(&self) -> &str {
        match self {
            Self::Left => "Left",
            Self::Centre => "Centre",
            Self::Right => "Right",
        }
    }

    pub const fn value(self) -> i32 {
        self as i32
    }

    pub fn values() -> Vec<Self> {
        vec![Self::Left, Self::Centre, Self::Right]
    }
}
