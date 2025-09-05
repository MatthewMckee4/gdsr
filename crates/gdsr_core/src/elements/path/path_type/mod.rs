#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PathType {
    #[default]
    Square = 0,
    Round = 1,
    Overlap = 2,
}

impl PathType {
    #[must_use]
    pub const fn new(value: i32) -> Self {
        match value {
            1 => Self::Round,
            2 => Self::Overlap,
            _ => Self::Square,
        }
    }

    #[must_use]
    pub const fn value(&self) -> u16 {
        *self as u16
    }

    #[must_use]
    pub fn values() -> Vec<Self> {
        vec![Self::Square, Self::Round, Self::Overlap]
    }
}
