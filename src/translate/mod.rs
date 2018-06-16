
mod test;

pub mod symtab;
mod calc;

// RISC-V Translator


use ::parser::*;
use ::arch::inst;
use ::encode;
use ::arch::inst::arg::*;
use ::arch::system;

use ::calc::*;

use self::symtab::*;
use self::calc::*;

#[derive(Debug)]
pub enum TranslateError<'a> {
    Format,
    InstArity,
    UnknownInst,
    UnknownSymbol(&'a Expr),
    UnknownCsrName(&'a Expr),
    CalcError(CalcError<'a>),
    NotImplemented(&'a Expr),
}

pub fn translate<'a>(expr: &'a Expr, symtab: &Symtab, pc: u32) -> Result<u32, TranslateError<'a>> {

    match expr {
        &Expr::List { exprs: ref exprs, .. } => {
            if exprs.is_empty() {
                return Err(TranslateError::Format)
            }

            let (inst, args) = read_inst(exprs, symtab, pc)?;
            let code = encode::encode(inst, &args);
            Ok(code)
        }
        other => panic!("what this?: {:?}", other),
    }
}

/*
TODO: Fix the strangeness
The following behavior on Value::Location+ArgType::Address will make sense only if the base of load/store is the same as memory load offset of the generated code

Maybe the linker will be able to patch base pointer register and relativeness of imm
so that this translate logic doesn't have to change...

Location symbol on ArgType::Address Arg will be relatively resolved.
location symbol otherwise evaluates to that location.
ArgType::Address otherwise will take the value as is.

*/
fn read_inst<'a>(exprs: &'a [Expr], symtab: &Symtab, pc: u32) -> Result<(&'static inst::Inst, Vec<u32>), TranslateError<'a>> {
    if exprs.is_empty() {
        return Err(TranslateError::Format);
    }

    // Determine inst
    let inst = match &exprs[0] {
        &Expr::Identifier { value: ref v, .. } => inst::inst(v).ok_or(TranslateError::UnknownInst)?,
        _ => return Err(TranslateError::Format),
    };

    let expr_rest = &exprs[1..];

    if inst.args.len() != expr_rest.len() {
        return Err(TranslateError::InstArity);
    }

    // Resolve symbols
    let mut args = Vec::<u32>::new();

    for (e, &a) in expr_rest.iter().zip(inst.args) {
        // For some reason, only identifier passed to csr argument is special
        match e {
            &Expr::Identifier { value: ref s, .. } => {
                match a.arg_type {
                    ArgType::Csr => {
                        match system::csr_value(s) {
                            Some(v) => args.push(v),
                            None => return Err(TranslateError::UnknownCsrName(e)),
                        }
                    }
                    _ => evaluate(e, symtab, &mut args)?,
                }
            }
            _ => evaluate(e, symtab, &mut args)?,
        }
    }

    // Generate args
    Ok((inst, args))
}

fn evaluate<'a>(expr: &'a Expr, symtab: &Symtab, args: &mut Vec<u32>) -> Result<(), TranslateError<'a>> {
    match calculate(expr, symtab) {
        Ok(v) => args.push(v as u32),
        Err(e) => return Err(TranslateError::CalcError(e)),
    }

    Ok(())
}
