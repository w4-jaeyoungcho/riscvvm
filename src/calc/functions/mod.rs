
#[cfg(test)]
mod test;

use ::backtrace::Backtrace;

#[derive(Debug)]
pub enum FunctionError {
    Arity,
    Overflow,
}

const FUNCTIONS: &[(&'static str, fn(&[i64]) -> Result<i64, FunctionError>)] = &[
    // These panic at overflow...
    // TODO: return error instead of panicking
    ("+", add),
    ("-", subtract),
    ("*", multiply),
    ("/", divide),
    ("%", remainder),

    // wrapping operators
    ("&+", overflow_add),
    ("&-", overflow_subtract),
    ("&*", overflow_multiply),

    // Bitwise operators
    ("~", bitwise_not),
    ("&", bitwise_and),
    ("|", bitwise_or),
    ("^", bitwise_xor),
    ("<<", shl),
    (">>", shr),

    // ...
    ("*4", words),
];

pub fn function(name: &str) -> Option<fn(&[i64]) -> Result<i64, FunctionError>> {
    for &(n, f) in FUNCTIONS {
        if n == name {
            return Some(f);
        }
    }

    None
}

// Functions

fn add(args: &[i64]) -> Result<i64, FunctionError> {
    Ok(args.into_iter().fold(0, |b, n| b + n))
}

fn subtract(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() < 1 {
        return Err(FunctionError::Arity);
    }

    let base = args[0];

    if args.len() == 1 {
        return Ok(-base);
    }

    Ok((&args[1..]).into_iter().fold(base, |b, n| b - n))
}

fn multiply(args: &[i64]) -> Result<i64, FunctionError> {
    Ok(args.into_iter().fold(1, |b, n| b*n))
}


fn divide(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() < 2 { // ok
        return Err(FunctionError::Arity);
    }

    let base = args[0];

    Ok((&args[1..]).into_iter().fold(base, |b, n| b/n))
}


fn remainder(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() < 2 {
        return Err(FunctionError::Arity);
    }

    let base = args[0];

    Ok((&args[1..]).into_iter().fold(base, |b, n| b%n))
}

// Overflow operators

fn overflow_add(args: &[i64]) -> Result<i64, FunctionError> {
    Ok(args.into_iter().fold(0, |b, &n| b.wrapping_add(n)))
}

fn overflow_subtract(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() < 1 {
        return Err(FunctionError::Arity);
    }

    let base = args[0];

    if args.len() == 1 {
        return Ok(base.wrapping_neg());
    }

    Ok((&args[1..]).into_iter().fold(base, |b, &n| b.wrapping_sub(n)))
}

fn overflow_multiply(args: &[i64]) -> Result<i64, FunctionError> {
    Ok(args.into_iter().fold(1, |b, &n| b.wrapping_mul(n)))
}


// Bitwise operators

fn bitwise_not(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() != 1 {
        return Err(FunctionError::Arity);
    }

    Ok(!args[0])
}

fn bitwise_and(args: &[i64]) -> Result<i64, FunctionError> {
    Ok((&args[..]).into_iter().fold(-1i64, |b, &n| b & n))
}

fn bitwise_or(args: &[i64]) -> Result<i64, FunctionError> {
    Ok((&args[..]).into_iter().fold(0, |b, &n| b | n))
}

fn bitwise_xor(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() != 2 {
        return Err(FunctionError::Arity);
    }

    Ok(args[0] ^ args[1])
}


fn shl(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() != 2 {
        return Err(FunctionError::Arity);
    }

    Ok(args[0] >> args[1])
}

fn shr(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() != 2 {
        return Err(FunctionError::Arity);
    }

    Ok(args[0] << args[1])
}

fn words(args: &[i64]) -> Result<i64, FunctionError> {
    if args.len() != 1 {
        return Err(FunctionError::Arity);
    }

    Ok(args[0] * 4)
}

/*
Template

fn multiply(args: &[i64]) -> Result<i64, FunctionError> {

}
*/