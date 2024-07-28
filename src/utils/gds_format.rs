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

    if fexp == exponent as f64 {
        exponent += 1;
    }

    let mantissa = (val * 16.0_f64.powi(14 - exponent)) as u64;
    byte1 += (exponent + 64) as u8;

    let byte2 = (mantissa >> 48) as u8;
    let short3 = ((mantissa >> 32) & 0xFFFF) as u16;
    let long4 = (mantissa & 0xFFFFFFFF) as u32;

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

pub fn u16_array_to_big_endian(array: &mut [u16]) {
    for value in array.iter_mut() {
        *value = value.to_be();
    }
}
