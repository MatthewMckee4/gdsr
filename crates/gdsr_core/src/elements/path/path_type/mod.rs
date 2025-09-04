#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum PathType {
    #[default]
    Square = 0,
    Round = 1,
    Overlap = 2,
}

impl PathType {
    pub fn new(value: i32) -> Self {
        match value {
            0 => PathType::Square,
            1 => PathType::Round,
            2 => PathType::Overlap,
            _ => PathType::Square,
        }
    }

    pub fn value(&self) -> i32 {
        *self as i32
    }

    pub fn values() -> Vec<PathType> {
        vec![PathType::Square, PathType::Round, PathType::Overlap]
    }
}
