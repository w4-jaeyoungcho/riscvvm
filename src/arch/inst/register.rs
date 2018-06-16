
// Abi names

// dynamic

pub fn abi_name(reg: u8) -> &'static str {
    match reg {
        0 => "zero",
        1 => "ra",
        2 => "sp",
        3 => "gp",
        4 => "tp",
        5 => "t0",
        6 => "t1",
        7 => "t2",
        8 => "s0",
        9 => "s1",
        10 => "a0", // return
        11 => "a1", // return
        12 => "a2", // why not return? ...
        13 => "a3",
        14 => "a4",
        15 => "a5",
        16 => "a6",
        17 => "a7",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "s8",
        25 => "s9",
        26 => "s10",
        27 => "s11",
        28 => "t3",
        29 => "t4",
        30 => "t5",
        31 => "t6",
        _ => panic!("abi_name with {}", reg),
    }
}

pub fn next_saved_register(r: u8) -> Option<u8> {
    assert_eq!(abi_name(r).chars().nth(0).unwrap(), 's', "not saved");

    if r == 9 {
        Some(18)
    } else if r == 27 {
        None
    } else {
        Some(r + 1)
    }
}

pub fn next_temp_register(r: u8) -> Option<u8> {
    assert_eq!(abi_name(r).chars().nth(0).unwrap(), 't', "not temp");

    if r == 7 {
        Some(28)
    } else if r == 31 {
        None
    } else {
        Some(r + 1)
    }
}

pub fn index(name: &str) -> Option<u8> {
    for i in 0..32 {
        if name == abi_name(i) {
            return Some(i);
        }
    }

    None
}
