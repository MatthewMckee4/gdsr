#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PathType {
    #[default]
    Square = 0,
    Round = 1,
    Overlap = 2,
}

impl std::fmt::Display for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PathType {}", self.name())
    }
}

impl std::fmt::Debug for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl PathType {
    pub fn new(value: i32) -> Result<Self, String> {
        match value {
            0 => Ok(PathType::Square),
            1 => Ok(PathType::Round),
            2 => Ok(PathType::Overlap),
            _ => Err("Invalid value for PathType".to_string()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            PathType::Square => "Square Ends",
            PathType::Round => "Round Ends",
            PathType::Overlap => "Overlap Ends",
        }
    }

    pub fn value(&self) -> i32 {
        *self as i32
    }

    pub fn values() -> Vec<PathType> {
        vec![PathType::Square, PathType::Round, PathType::Overlap]
    }
}
