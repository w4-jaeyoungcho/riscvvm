#![cfg(test)]

use super::*;

#[test]
fn test_array_to_u32() {
    let a = [0x12u8, 0x34, 0x56, 0x78];

    let x = array_to_u32(&a);
    assert_eq!(x, 0x78563412);
}