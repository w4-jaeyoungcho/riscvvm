
use super::*;

#[test]
fn test_add() {
    let args: [i64; 3] = [2, 7, 1];

    match add(&args[..]) {
        Ok(v) => assert_eq!(v, 10, "add"),
        Err(e) => panic!(e),
    }
}

#[test]
fn test_subtract() {
    let args = [2i64, 7, 1];

    match subtract(&args[..]) {
        Ok(v) => assert_eq!(v, -6, "subtract"),
        Err(e) => panic!(e),
    }


    match subtract(&[2i64]) {
        Ok(v) => assert_eq!(v, -2, "subtract"),
        Err(e) => panic!(e),
    }
}

#[test]
fn test_multiply() {
    let args = [2i64, 7, 1];

    match multiply(&args[..]) {
        Ok(v) => assert_eq!(v, 14, "multiply"),
        Err(e) => panic!(e),
    }
}

#[test]
fn test_divide() {
    let args = [12i64, 3, 2];

    match divide(&args[..]) {
        Ok(v) => assert_eq!(v, 2, "divide"),
        Err(e) => panic!(e),
    }
}

#[test]
fn test_remainder() {
    let args = [12i64, 5];

    match remainder(&args[..]) {
        Ok(v) => assert_eq!(v, 2, "remainder"),
        Err(e) => panic!(e),
    }
}
