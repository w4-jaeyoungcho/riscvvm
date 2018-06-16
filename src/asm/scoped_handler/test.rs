
use super::*;

#[test]
fn test_u32_to_array() {
    assert_eq!(u32_to_array(0x12345678), [0x78u8, 0x56, 0x34, 0x12]);
}