/// Round a floating point value to a specified number of decimal places.
///
/// Multiplies by `10^ndigits`, rounds, then divides back.
pub fn round_to_decimals(value: f64, ndigits: u32) -> f64 {
    let factor = 10f64.powi(ndigits as i32);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_round_to_decimals() {
        assert_eq!(round_to_decimals(3.14159, 2), 3.14);
        assert_eq!(round_to_decimals(3.14159, 3), 3.142);
        assert_eq!(round_to_decimals(3.14159, 0), 3.0);
        assert_eq!(round_to_decimals(-2.5678, 2), -2.57);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    /// Rounding to 0 decimals should produce an integer value.
    #[quickcheck]
    fn round_to_zero_decimals_is_integer(v: i16) -> bool {
        let f = f64::from(v) + 0.3;
        let r = round_to_decimals(f, 0);
        r == r.round()
    }
}
