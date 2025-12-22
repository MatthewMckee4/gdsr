pub fn eight_byte_real(value: f64) -> [u8; 8] {
    if value == 0.0 {
        return [0x00; 8];
    }

    let mut byte1: u8;
    let mut val = value;

    if val < 0.0 {
        byte1 = 0x80;
        val = -val;
    } else {
        byte1 = 0x00;
    }

    let fexp = val.log2() / 4.0;
    let mut exponent = fexp.ceil() as i32;

    if fexp == f64::from(exponent) {
        exponent += 1;
    }

    let mantissa = (val * 16.0_f64.powi(14 - exponent)) as u64;
    byte1 += (exponent + 64) as u8;

    let byte2 = (mantissa >> 48) as u8;
    let short3 = ((mantissa >> 32) & 0xFFFF) as u16;
    let long4 = (mantissa & 0xFFFF_FFFF) as u32;

    let mut result = [0u8; 8];
    result[0] = byte1;
    result[1] = byte2;
    result[2] = (short3 >> 8) as u8;
    result[3] = (short3 & 0xFF) as u8;
    result[4] = (long4 >> 24) as u8;
    result[5] = (long4 >> 16) as u8;
    result[6] = (long4 >> 8) as u8;
    result[7] = (long4 & 0xFF) as u8;

    result
}

pub fn u16_array_to_big_endian(array: &[u16]) -> Vec<u16> {
    let mut result = Vec::with_capacity(array.len());
    for value in array {
        result.push(value.to_be());
    }
    result
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_eight_byte_real_zero() {
        let value = 0.0;
        let result = eight_byte_real(value);

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
        let result = eight_byte_real(value);

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
        let result = eight_byte_real(value);

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
        let result = eight_byte_real(value);

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
}
