
#[cfg(test)]
mod test;

use super::*;
use ::arch::inst;
use ::parser::*;
use ::translate::symtab::Symtab;

use ::translate::*;

use std::io::prelude::*;


pub struct Phase<'a> {
    writer: &'a mut Write,
}

impl<'a> Phase<'a> {
    pub fn new(writer: &'a mut Write) -> Phase<'a> {
        Phase {
            writer: writer,
        }
    }
}

impl<'a> ScopedHandler for Phase<'a> {
    fn handle(&mut self, expr: &Expr, symtab: &mut Symtab, counter: u32) -> Result<u32, AsmProcessError> {
        let op = get_op(expr)?;

        // Handle label
        if op == ":" {
            let args = get_args(expr);
            if args.len() != 1 {
                return Err(AsmProcessError::DirectiveFormat);
            }
            if let &Expr::Identifier { value: ref s, .. } = &args[0] {
                // registering the label
                symtab.register_label(&s, counter).map_err(|e| AsmProcessError::Symtab(e))?;
            } else {
                return Err(AsmProcessError::DirectiveFormat);
            }

            return Ok(0);
        }

        // Handle .equ
        if op == ".equ" {
            // handle equ
            let args = get_args(expr);
            if args.len() != 2 {
                return Err(AsmProcessError::DirectiveFormat);
            }

            let id = args[0].get_identifier().ok_or(AsmProcessError::DirectiveFormat)?;
            let v = args[1].get_integer().ok_or(AsmProcessError::DirectiveFormat)?;

            let sym = Symbol {
                ext: false,
                mutable: false,
                value: Value::General(v),
            };
            symtab.insert(id, sym).map_err(|e| AsmProcessError::Symtab(e))?;

            return Ok(0);
        }

        if op == ".def" {
            // handle
            let args = get_args(expr);
            if args.len() != 2 {
                return Err(AsmProcessError::DirectiveFormat);
            }

            let id = args[0].get_identifier().ok_or(AsmProcessError::DirectiveFormat)?;
            let v = args[1].get_identifier().ok_or(AsmProcessError::DirectiveFormat)?;

            let reg = ::arch::inst::register::index(v).ok_or(AsmProcessError::DirectiveFormat)?;

            let sym = Symbol {
                ext: false,
                mutable: false,
                value: Value::Register(reg),
            };
            symtab.insert(id, sym).map_err(|e| AsmProcessError::Symtab(e))?;

            return Ok(0);
        }

        // Other unhandled directives...
        if !inst::is_inst(op) {
//            return Ok(increment(expr).unwrap());
            return Err(AsmProcessError::UnknownDirective(String::from(op)));
        }

        // Translate and write
        // update symbol pc to the current counter
        symtab.update("pc", Symbol {
            ext: false,
            mutable: false,
            value: Value::Location(counter),
        });

        let code = translate(expr, symtab, counter).map_err(|e| {
            let s = format!("{:?}", e);
            AsmProcessError::Translate(s)
        })?;

        // write
        let buf = u32_to_array(code);
        let written = self.writer.write(&buf).map_err(|e| AsmProcessError::IO(e))?;
        if written != 4 {
            return Err(AsmProcessError::Unknown);
        }

        // wrote

        Ok(4)
    }

    fn org(&mut self, counter: u32) -> Result<(), AsmProcessError> {
        // TODO can't be handled like this because Writer is not seekable you idiot
        panic!("Handle me")
    }
}

fn u32_to_array(mut x: u32) -> [u8; 4] {
    let mut a = [0u8; 4];
    a[0] = x as u8;
    x >>= 8;
    a[1] = x as u8;
    x >>= 8;
    a[2] = x as u8;
    x >>= 8;
    a[3] = x as u8;

    a
}

fn get_args<'a>(expr: &'a Expr) -> &'a [Expr] {
    if let &Expr::List { exprs: ref exprs, .. } = expr {
        if exprs.is_empty() {
            panic!("not directive")
        }

        &exprs[1..]
    } else {
        panic!("not list")
    }
}