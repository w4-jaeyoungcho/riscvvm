
pub fn write(base: u32, offset: u8, length: u8, value: u32) -> u32 {
    base | ((((1u32 << length) - 1) & value) << offset)
}

pub fn read(base: u32, offset: u8, length: u8) -> u32 {
    (base >> offset) & ((1u32 << length) - 1)
}

pub fn sign_extend(x: u32, sign_bit: u8) -> u32 {
    let s = (x  >> sign_bit) & 0x1;

    if s == 1 {
        let rest_bits = 31 - sign_bit;
        ((x << rest_bits) as i32 >> rest_bits) as u32
    } else {
        x
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sign_extend() {
        let x = 0x123u32;
        assert_eq!(sign_extend(x, 8), 0xFFFFFF23);
        assert_eq!(sign_extend(x, 9), 0x00000123);
    }
}