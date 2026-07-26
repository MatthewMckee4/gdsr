use std::fmt;

/// Smallest positive normalized GDS REAL8 value, `16^-65`.
pub(super) const MIN_GDS_REAL8_MAGNITUDE: f64 = f64::from_bits(763_u64 << 52);
/// Exclusive upper magnitude bound for GDS REAL8, `16^63`.
pub(super) const MAX_GDS_REAL8_MAGNITUDE: f64 = f64::from_bits(1275_u64 << 52);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GdsReal8Error {
    NonFinite,
    MagnitudeTooSmall,
    MagnitudeTooLarge,
}

impl fmt::Display for GdsReal8Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => formatter.write_str("value must be finite"),
            Self::MagnitudeTooSmall => write!(
                formatter,
                "nonzero magnitude must be at least {MIN_GDS_REAL8_MAGNITUDE}"
            ),
            Self::MagnitudeTooLarge => write!(
                formatter,
                "magnitude must be less than {MAX_GDS_REAL8_MAGNITUDE}"
            ),
        }
    }
}

#[inline]
pub(super) fn eight_byte_real(value: f64) -> Result<[u8; 8], GdsReal8Error> {
    if !value.is_finite() {
        return Err(GdsReal8Error::NonFinite);
    }
    if value == 0.0 {
        return Ok([0x00; 8]);
    }

    let magnitude = value.abs();
    if magnitude < MIN_GDS_REAL8_MAGNITUDE {
        return Err(GdsReal8Error::MagnitudeTooSmall);
    }
    if magnitude >= MAX_GDS_REAL8_MAGNITUDE {
        return Err(GdsReal8Error::MagnitudeTooLarge);
    }

    let bits = magnitude.to_bits();
    let binary_exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    let exponent = binary_exponent.div_euclid(4) + 1;
    let encoded_exponent = (exponent + 64) as u8;
    let significand = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
    let mantissa = significand << binary_exponent.rem_euclid(4);
    let sign = if value.is_sign_negative() { 0x80 } else { 0 };
    let mut result = [0u8; 8];
    result[0] = sign | encoded_exponent;
    result[1..].copy_from_slice(&mantissa.to_be_bytes()[1..]);
    Ok(result)
}

pub fn write_u16_array_as_big_endian(
    buffer: &mut impl std::io::Write,
    array: &[u16],
) -> std::io::Result<()> {
    for value in array {
        buffer.write_all(&value.to_be_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_eight_byte_real_zero() {
        let value = 0.0;
        let result = eight_byte_real(value).expect("zero should be representable");

        assert_debug_snapshot!(result, @r"
        [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]
        ");
    }

    #[test]
    fn test_eight_byte_real_negative() {
        let value = -123.456;
        let result = eight_byte_real(value).expect("value should be representable");

        assert_debug_snapshot!(result, @r"
        [
            194,
            123,
            116,
            188,
            106,
            126,
            249,
            220,
        ]
        ");
    }

    #[test]
    fn test_eight_byte_real_positive() {
        let value = 123.456;
        let result = eight_byte_real(value).expect("value should be representable");

        assert_debug_snapshot!(result, @r"
        [
            66,
            123,
            116,
            188,
            106,
            126,
            249,
            220,
        ]
        ");
    }

    #[test]
    fn test_height_byte_real_log() {
        let value = 16.0;
        let result = eight_byte_real(value).expect("value should be representable");

        assert_debug_snapshot!(result, @r"
        [
            66,
            16,
            0,
            0,
            0,
            0,
            0,
            0,
        ]
        ");
    }

    #[test]
    fn eight_byte_real_checks_format_boundaries() {
        assert!(eight_byte_real(MIN_GDS_REAL8_MAGNITUDE).is_ok());
        assert!(eight_byte_real(f64::from_bits(MIN_GDS_REAL8_MAGNITUDE.to_bits() - 1)).is_err());
        assert!(eight_byte_real(f64::from_bits(MAX_GDS_REAL8_MAGNITUDE.to_bits() - 1)).is_ok());
        assert!(eight_byte_real(MAX_GDS_REAL8_MAGNITUDE).is_err());
        assert!(eight_byte_real(f64::INFINITY).is_err());
        assert!(eight_byte_real(f64::NAN).is_err());
    }
}
