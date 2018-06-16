
// RISC-V Assembler

/*

## levels of ASM

phase 1
* scanStatements - ordering of input - handle include commands
* see what it is
* collect symbol, location from labels

phase 2
* scanStatements
* see what it is - statements - directives
* handle asm directives
  * def
  * undef
  * equ
* evaluate symbols and arithmetic expressions
* encode

## Syntax/Directives

### scan
(include "banana.asm")
(label here)
(org 0x00000002)

###
(label banana) (bytes 4)
(label strawberry) (dw 0x02)
(equ output 0x00100000)

## Sample

### asm/high-level mixup

(var count)
(var first 10)

(func fibonacci (n) (res)
    (set res 1)
    (if (= n 0) END)
    (if (= n 1) END)
    (set res (+ (fibonacci (- n 1)) (fibonacci (- n 2)))
    (label END))

(func gcd (a b) (res)
    (var temp)
    (if (> a b) SKIP_SWAP)
    (set (a b) (values b a))
    (label SKIP_SWAP)

    (label LOOP_START)
    (if (= b 1) END)
    (set (a b) (values b (% a b))
    (label END)

    (set res a))

*/

mod scoped_handler;
mod test;
mod scoper;

use std;
use std::io::prelude::*;
use ::translate::*;

use ::translate::symtab::*;

use ::parser::*;

use self::scoped_handler::*;
use self::scoper::*;

use ::arch::inst;

#[derive(Debug)]
pub struct AsmError {
    pub file_path: String,
    pub line_index: usize,
    pub column_index: usize,
    pub error: AsmProcessError,
}

impl AsmError {
    fn from_expr(file_path: &str, expr: &Expr, error: AsmProcessError) -> AsmError {
        let token = expr.token();

        AsmError {
            file_path: String::from(file_path),
            line_index: token.line_index,
            column_index: token.column_index,
            error: error,
        }
    }
}

#[derive(Debug)]
pub enum AsmProcessError {
    Translate(String),
    Parser(String),
    Symtab(SymtabError),
    UnknownDirective(String),
    DirectiveFormat,
    BlockFormat,
    File(std::io::Error),
    IO(std::io::Error),
    FromInclude(Box<AsmError>),
    CalcError(String),
    Unknown,
}

trait WalkHandler {
    fn handle(&mut self, expr: &Expr, counter: u32) -> Result<u32, AsmProcessError>;

    fn enter_scope(&mut self, scope_name: &Expr, file_name: &str, counter: u32) -> Result<u32, AsmProcessError>;
    fn exit_scope(&mut self, counter: u32) -> Result<u32, AsmProcessError>;

    fn org(&mut self, counter: u32) -> Result<(), AsmProcessError>;
}

trait ScopedHandler {
    fn handle(&mut self, expr: &Expr, symtab: &mut Symtab, counter: u32) -> Result<u32, AsmProcessError>;

    fn org(&mut self, counter: u32) -> Result<(), AsmProcessError>;
}

use std::io::Write;

struct NullWriter;

impl Write for NullWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

// file_path better be openable.
pub fn assemble<R: Read, W: Write, F>(mut writer: W, input_file: &str, get_reader_func: F) -> Result<(), AsmError>
    where F: Fn(&str) -> Result<R, std::io::Error> {

    // Prepare symtab
    let mut symtab = Symtab::prepopulated();

    symtab.placeholder_symbol = true;
    symtab.allow_update = false;

    // Phase 1 : Build symtab
    {
        println!("Phase 1 start");

        let reader = get_reader_func(input_file).unwrap();

        println!("File opened: {}", input_file);

        let mut null_writer = NullWriter;
        let mut scoped_handler = Phase::new(&mut null_writer);

        let mut walk_handler = Scoper::new(&mut symtab, &mut scoped_handler);

        let mut walker = Walker::new(&get_reader_func, &mut walk_handler);

        walker.walk(input_file, reader)?;

        println!("Phase 1 done: counter={}", walker.counter);
        println!();
    }

    symtab.placeholder_symbol = false;
    symtab.allow_update = true;

    symtab.print();

    // Phase 2 : Encode
    {
        println!("Phase 2 start");

        let reader = get_reader_func(input_file).unwrap();

        println!("File opened: {}", input_file);

        let mut scoped_handler = Phase::new(&mut writer);

        let mut walk_handler = Scoper::new(&mut symtab, &mut scoped_handler);

        let mut walker = Walker::new(&get_reader_func, &mut walk_handler);

        walker.walk(input_file, reader)?;

        println!("Phase 2 done: counter={}", walker.counter);
    }

    println!("Flushing");
    writer.flush().unwrap();
    println!("Successfully wrote!");

    Ok(())
}

// wrapper for assemble with memory as IO
pub fn assemble_mem(input: &str) -> Result<Vec<u8>, AsmError> {
    use ::lexer::mem_reader;

    let mut writer: Vec<u8> = Vec::new();

    assemble(&mut writer, "memory", |file_path| { match file_path {
        "memory" => Ok(Box::new(mem_reader::MemReader::new(input.as_bytes()))),
        _ => panic!("assemble_mem with include {}", file_path),
    } })?;

    Ok(writer)
}

// Function tentative
// Walker has to know counter increment of every directive/instruction. Counter is fully managed by
struct Walker<'a, R: Read, H: WalkHandler + 'a, F: Fn(&str) -> Result<R, std::io::Error>> {
    counter: u32,
    get_reader_func: F,
    handler: &'a mut H,
}

impl<'a, R: Read, H: WalkHandler, F: Fn(&str) -> Result<R, std::io::Error>> Walker<'a, R, H, F> {
    fn new(get_reader_func: F, handler: &'a mut H) -> Walker<'a, R, H, F> {
        Walker {
            counter: 0,
            get_reader_func: get_reader_func,
            handler: handler,
        }
    }

    fn handle_statement(&mut self, expr: &Expr, input_file: &str) -> Result<(), AsmError> {
        //        println!("walk on {:?}", result);

        // Check if include
        let mut handled = false;

        match expr {
            &Expr::List { exprs: ref exprs, .. } => {
                if let Some(op_expr) = exprs.get(0) {
                    if let &Expr::Identifier { value: ref value, .. } = op_expr {
                        match value.as_str() {
                            ".include" => {
                                // Handle include
                                if exprs.len() != 2 {
                                    return Err(AsmError::from_expr(input_file, &exprs[0], AsmProcessError::DirectiveFormat));
                                }

                                let include_target = if let &Expr::String { value: ref s, .. } = &exprs[1] {
                                    s
                                } else {
                                    return Err(AsmError::from_expr(input_file, &exprs[1], AsmProcessError::DirectiveFormat));
                                };

                                let included = (self.get_reader_func)(include_target).map_err(|e| {
                                    AsmError::from_expr(input_file, &exprs[1], AsmProcessError::File(e))
                                })?;

                                self.walk(include_target, included).map_err(|e| {
                                    let child_error = AsmProcessError::FromInclude(Box::new(e));
                                    AsmError::from_expr(input_file, &expr, child_error)
                                })?;
                                handled = true;
                            }
                            ".org" => panic!("Handle me!"),
                            ".begin" | ".block" | ".func" | ".leaf" => {
                                let inc = self.handler.enter_scope(op_expr, input_file, self.counter).map_err(|e| AsmError::from_expr(input_file, &expr, e))?;
                                self.counter += inc;

                                self.walk_scope(&exprs[1..], input_file)?;

                                let inc = self.handler.exit_scope(self.counter).map_err(|e| AsmError::from_expr(input_file, &expr, e))?;
                                self.counter += inc;

                                handled = true;
                            }
                            _ => (),
                        }
                    }
                }
            }
            _ => {}
        }

        if !handled {
            let inc = self.handler.handle(&expr, self.counter).map_err(|e| AsmError::from_expr(input_file, &expr, e))?;

            self.counter += inc;
        }

        Ok(())
    }

    fn walk(&mut self, input_file: &str, reader: R) -> Result<(), AsmError> {
        // Handle includes and wrap process error
        println!("walk_statement: handling file {}", input_file);

        let mut parser = Parser::new(reader);

        while let Some(result) = parser.next() {
            let expr = result.map_err(|error| {
                let pos = error.pos();

                AsmError {
                    file_path: String::from(input_file),
                    line_index: pos.0,
                    column_index: pos.1,
                    error: AsmProcessError::Parser(format!("{:?}", error)),
                }
            })?;

            self.handle_statement(&expr, input_file)?;
        }

        println!("walk_statement: finish handling file {} counter={}", input_file, self.counter);
        Ok(())
    }

    fn walk_scope(&mut self, exprs: &[Expr], input_file: &str) -> Result<(), AsmError> {
        for expr in exprs {
            self.handle_statement(expr, input_file)?;
        }
        Ok(())
    }
}

/*
Directives
non-advancing
* def
* undef
* equ
* label

advancing
* (bytes <n>)
* (dw word1 word2 ...) -> 4 * n
* db
* dh

A logical instruction may have any increment.
But let's fix it to 4
*/

fn get_op<'a>(expr: &'a Expr) -> Result<&'a str, AsmProcessError> {
    if let &Expr::List { exprs: ref exprs, .. } = expr {
        if exprs.is_empty() {
            return Err(AsmProcessError::DirectiveFormat);
        }

        if let &Expr::Identifier { value: ref s, .. } = &exprs[0] {
            Ok(s.as_str())
        } else {
            return Err(AsmProcessError::DirectiveFormat);
        }

    } else {
        return Err(AsmProcessError::DirectiveFormat);
    }
}
