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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_type_new() {
        assert_eq!(PathType::new(0), PathType::Square);
        assert_eq!(PathType::new(1), PathType::Round);
        assert_eq!(PathType::new(2), PathType::Overlap);
        assert_eq!(PathType::new(-1), PathType::Square);
        assert_eq!(PathType::new(999), PathType::Square);
    }

    #[test]
    fn test_path_type_value() {
        assert_eq!(PathType::Square.value(), 0);
        assert_eq!(PathType::Round.value(), 1);
        assert_eq!(PathType::Overlap.value(), 2);
    }

    #[test]
    fn test_path_type_values() {
        let values = PathType::values();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&PathType::Square));
        assert!(values.contains(&PathType::Round));
        assert!(values.contains(&PathType::Overlap));
    }

    #[test]
    fn test_path_type_default() {
        assert_eq!(PathType::default(), PathType::Square);
    }

    #[test]
    fn test_path_type_debug() {
        assert_eq!(format!("{:?}", PathType::Square), "Square");
        assert_eq!(format!("{:?}", PathType::Round), "Round");
        assert_eq!(format!("{:?}", PathType::Overlap), "Overlap");
    }

    #[test]
    fn test_path_type_clone_and_copy() {
        let path_type = PathType::Round;
        let cloned = path_type;
        let copied = path_type;

        assert_eq!(path_type, cloned);
        assert_eq!(path_type, copied);
    }

    #[test]
    fn test_path_type_partial_eq() {
        assert_eq!(PathType::Square, PathType::Square);
        assert_ne!(PathType::Square, PathType::Round);
        assert_ne!(PathType::Round, PathType::Overlap);
    }
}
