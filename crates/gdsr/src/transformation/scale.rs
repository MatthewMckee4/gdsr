use crate::Point;

#[derive(Clone, Debug, PartialEq)]
pub enum ScaleError {
    Zero,
    Negative(f64),
    NaN,
    Infinite,
}

impl std::fmt::Display for ScaleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "Scale factor must not be zero"),
            Self::Negative(v) => write!(f, "Scale factor must be positive, got {v}"),
            Self::NaN => write!(f, "Scale factor must not be NaN"),
            Self::Infinite => write!(f, "Scale factor must be finite"),
        }
    }
}

impl std::error::Error for ScaleError {}

#[derive(Clone, Debug, PartialEq)]
pub struct Scale {
    factor: f64,
    centre: Point,
}

impl std::fmt::Display for Scale {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Scale by {} about {}", self.factor, self.centre)
    }
}

impl Scale {
    pub fn new(factor: f64, centre: Point) -> Result<Self, ScaleError> {
        if factor.is_nan() {
            return Err(ScaleError::NaN);
        }
        if factor.is_infinite() {
            return Err(ScaleError::Infinite);
        }
        if factor == 0.0 {
            return Err(ScaleError::Zero);
        }
        if factor < 0.0 {
            return Err(ScaleError::Negative(factor));
        }
        Ok(Self { factor, centre })
    }

    pub const fn factor(&self) -> f64 {
        self.factor
    }

    pub const fn centre(&self) -> &Point {
        &self.centre
    }

    pub fn apply_to_point(&self, point: &Point) -> Point {
        let self_center_x = self.centre.x();
        let self_center_y = self.centre.y();

        let dx = point.x() - self_center_x;
        let dy = point.y() - self_center_y;

        let new_x = (dx * self.factor) + self_center_x;
        let new_y = (dy * self.factor) + self_center_y;

        Point::new(new_x, new_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_new() {
        let scale = Scale::new(2.0, Point::integer(10, 20, 1e-9)).unwrap();
        assert_eq!(scale.factor(), 2.0);
        assert_eq!(scale.centre(), &Point::integer(10, 20, 1e-9));
    }

    #[test]
    fn test_scale_getters() {
        let centre = Point::integer(5, 10, 1e-9);
        let scale = Scale::new(1.5, centre).unwrap();
        assert_eq!(scale.factor(), 1.5);
        assert_eq!(scale.centre(), &centre);
    }

    #[test]
    fn test_scale_clone() {
        let scale = Scale::new(3.0, Point::integer(5, 5, 1e-9)).unwrap();
        let cloned = scale.clone();
        assert_eq!(scale, cloned);
    }

    #[test]
    fn test_scale_apply_at_centre() {
        let centre = Point::integer(10, 10, 1e-9);
        let scale = Scale::new(2.0, centre).unwrap();
        let scaled = scale.apply_to_point(&centre);

        // Point at centre should remain unchanged
        assert_eq!(scaled, centre);
    }

    #[test]
    fn test_scale_double() {
        let scale = Scale::new(2.0, Point::integer(0, 0, 1e-9)).unwrap();
        let point = Point::integer(10, 5, 1e-9);
        let scaled = scale.apply_to_point(&point);

        let expected = Point::integer(20, 10, 1e-9);
        assert_eq!(scaled.x(), expected.x());
        assert_eq!(scaled.y(), expected.y());
    }

    #[test]
    fn test_scale_display() {
        let scale = Scale::new(2.5, Point::integer(10, 20, 1e-9)).unwrap();
        let display_str = format!("{scale}");
        assert!(display_str.contains("Scale by 2.5 about"));
        assert!(display_str.contains("Point("));
    }

    #[test]
    fn test_scale_half() {
        let scale = Scale::new(0.5, Point::integer(0, 0, 1e-9)).unwrap();
        let point = Point::integer(20, 10, 1e-9);
        let scaled = scale.apply_to_point(&point);

        let expected = Point::integer(10, 5, 1e-9);
        assert_eq!(scaled.x(), expected.x());
        assert_eq!(scaled.y(), expected.y());
    }

    #[test]
    fn test_scale_valid_factor() {
        assert!(Scale::new(1.0, Point::integer(0, 0, 1e-9)).is_ok());
        assert!(Scale::new(0.5, Point::integer(0, 0, 1e-9)).is_ok());
        assert!(Scale::new(100.0, Point::integer(0, 0, 1e-9)).is_ok());
    }

    #[test]
    fn test_scale_zero_factor() {
        let result = Scale::new(0.0, Point::integer(0, 0, 1e-9));
        assert_eq!(result, Err(ScaleError::Zero));
    }

    #[test]
    fn test_scale_negative_factor() {
        let result = Scale::new(-1.0, Point::integer(0, 0, 1e-9));
        assert_eq!(result, Err(ScaleError::Negative(-1.0)));
    }

    #[test]
    fn test_scale_nan_factor() {
        let result = Scale::new(f64::NAN, Point::integer(0, 0, 1e-9));
        assert_eq!(result, Err(ScaleError::NaN));
    }

    #[test]
    fn test_scale_infinity_factor() {
        let result = Scale::new(f64::INFINITY, Point::integer(0, 0, 1e-9));
        assert_eq!(result, Err(ScaleError::Infinite));

        let result = Scale::new(f64::NEG_INFINITY, Point::integer(0, 0, 1e-9));
        assert_eq!(result, Err(ScaleError::Infinite));
    }

    #[test]
    fn test_scale_error_display() {
        assert_eq!(
            ScaleError::Zero.to_string(),
            "Scale factor must not be zero"
        );
        assert_eq!(
            ScaleError::Negative(-2.0).to_string(),
            "Scale factor must be positive, got -2"
        );
        assert_eq!(ScaleError::NaN.to_string(), "Scale factor must not be NaN");
        assert_eq!(
            ScaleError::Infinite.to_string(),
            "Scale factor must be finite"
        );
    }
}
