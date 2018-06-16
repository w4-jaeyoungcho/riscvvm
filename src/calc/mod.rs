#[cfg(test)]
mod test;

mod functions;

use ::parser::Expr;
use ::translate::symtab::Symtab;

use self::functions::*;

#[derive(Debug)]
pub enum CalcError<'a> {
    UnknownSymbol(&'a Expr),
    UnknownFunction(&'a Expr),
    FormatError(&'a Expr),
    // Got '()'
    FunctionError(FunctionError),
}

pub fn calculate<'a>(expr: &'a Expr, symtab: &Symtab) -> Result<i64, CalcError<'a>> {
    match expr {
        &Expr::Identifier { value: ref value, .. } => {
            // Look up
            match symtab.get(value) {
                Some(symbol) => Ok(symbol.value.eval()),
                None => Err(CalcError::UnknownSymbol(expr)),
            }
        }
        &Expr::Integer { value: value, .. } => {
            Ok(value)
        }
        &Expr::String { .. } => {
            // ...
            panic!("calculate cannot yet handle strings: {:?}", expr);
        }
        &Expr::List { exprs: ref exprs, .. } => {
            // Function invocation

            if exprs.is_empty() {
                return Err(CalcError::FormatError(expr));
            }

            let function_name: &str = match &exprs[0] {
                &Expr::Identifier { value: ref value, .. } => {
                    value
                }
                _ => return Err(CalcError::FormatError(expr)),
            };

            let f = function(function_name).ok_or(CalcError::UnknownFunction(expr))?;

            // Recurse evaluate args
            let mut args = Vec::<i64>::new();
            for e in &exprs[1..] {
                match calculate(e, symtab) {
                    Ok(v) => args.push(v),
                    Err(e) => return Err(e)
                }
            }

            // function
            match f(&args[..]) {
                Ok(v) => Ok(v),
                Err(e) => Err(CalcError::FunctionError(e))
            }
        }
    }
}
