#![cfg_attr(not(test), no_std)]

pub fn split_u16_to_u8(number: u16) -> [u8; 2] {
    let high_byte = (number >> 8) as u8;
    let low_byte = (number & 0xFF) as u8;
    [high_byte, low_byte]
}

#[cfg(test)]
mod test {
    #[test]
    fn test_line_types() {}
}
