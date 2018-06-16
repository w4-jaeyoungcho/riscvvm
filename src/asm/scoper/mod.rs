
//! Implement a WalkHandler that wraps ScopedHandler
//! handles directives:
//! * .begin
//! * .func
//! When these scoping exprs are encountered,
//!

use super::*;

use ::arch::inst::register;
use ::parser;
use ::calc;

pub(super) struct Scoper<'s, 'h, H: ScopedHandler + 'h> {
    symtab: &'s mut Symtab,
    handler: &'h mut H,
}

impl<'s, 'h, H: ScopedHandler> Scoper<'s, 'h, H> {
    pub fn new(symtab: &'s mut Symtab, handler: &'h mut H) -> Scoper<'s, 'h, H> {
        Scoper {
            symtab: symtab,
            handler: handler,
        }
    }

    fn insert_func_prologue(&mut self, counter: u32) -> Result<u32, AsmProcessError> {
        // Inject function prologue
        /*
        (sw sp ra -4)
        (sw sp s0 -8)
        ...
        (addi sp sp -<stack frame size>)

        */

        let mut inc = 0u32;

        let saved_registers = self.symtab.get_immediate(".FUNC_SAVED_REGISTERS").unwrap().value.get_integers().clone();
        let stack_allocated = self.symtab.get_immediate(".FUNC_STACK_ALLOCATED").unwrap().value.eval();

        // Stack frame is multiple of 8 bytes why not
        let stack_frame_size = 8 * (((saved_registers.len() as i64 + 2) * 4 + stack_allocated + 7) / 8);

        self.symtab.insert(".FUNC_STACK_FRAME_SIZE", Symbol::new(Value::General(stack_frame_size as i64)));

        inc += self.handler.handle(&parser::parse("(sw sp ra -4)"), self.symtab, counter + inc).unwrap();
        for i in 0..saved_registers.len() {
            let reg = saved_registers[i];
            let offset = (i as i32 + 2) * -4;
            let line = format!("(sw sp {} {})", reg, offset);

            inc += self.handler.handle(&parser::parse(&line), self.symtab, counter + inc).unwrap();
        }

        inc += self.handler.handle(&parser::parse(&format!("(addi sp sp {})", -stack_frame_size)), self.symtab, counter + inc).unwrap();

        Ok(inc)
    }

    fn finish_func_preamble(&mut self, counter: u32) -> Result<u32, AsmProcessError> {
        let mut inc = 0u32;

        match self.symtab.get_immediate(".FUNC_PREAMBLE") {
            Some(_) => {
                let func_preamble = self.symtab.get_immediate(".FUNC_PREAMBLE").unwrap().value.eval();
                if func_preamble == 1 {
                    // func prologue not yet injected
                    inc += self.insert_func_prologue(counter + inc)?;
                    self.symtab.update(".FUNC_PREAMBLE", Symbol::new(Value::General(0)));
                }
            }
            None => ()
        }

        Ok(inc)
    }
}

impl<'s, 'h, H: ScopedHandler> WalkHandler for Scoper<'s, 'h, H> {

    fn handle(&mut self, expr: &Expr, counter: u32) -> Result<u32, AsmProcessError> {
        let mut inc = 0u32;

        let is_block_preamble = match self.symtab.get_immediate(".BLOCK_PREAMBLE") {
            Some(s) => s.value.eval() == 1,
            None => false,
        };

        if is_block_preamble {
            let block_name = expr.get_identifier().ok_or(AsmProcessError::BlockFormat)?;

            self.symtab.insert(".BLOCK_NAME", Symbol::new(Value::String(String::from(block_name))));
            self.symtab.update(".BLOCK_PREAMBLE", Symbol::new(Value::General(0)));

            self.symtab.insert(&(String::from(block_name) + ".BEGIN"), Symbol::new(Value::Location(counter)));

            return Ok(0);
        }

        let is_in_func_preamble = match self.symtab.get_immediate(".FUNC_PREAMBLE") {
            Some(s) => s.value.eval() == 1,
            None => false,
        };
        if is_in_func_preamble {
            /*
            To handle:
            * (.var n)
            * (.temp saved)
            * (.stack arr1 (* 10 4))

            */
            match expr {
                &Expr::List { exprs: ref exprs, .. } => {
                    if exprs.len() >= 2 {
                        match &exprs[0] {
                            &Expr::Identifier { value: ref value, .. } => {
                                if value == ".var" {
                                    if exprs.len() != 2 {
                                        return Err(AsmProcessError::DirectiveFormat);
                                    }

                                    let arg = exprs[1].get_identifier().ok_or(AsmProcessError::DirectiveFormat)?;

                                    let mut current_saved = self.symtab.get_immediate(".FUNC_SAVED_REGISTERS").unwrap().value.get_integers().clone();
                                    let next_saved_reg = current_saved.last()
                                        .map(|r| register::next_saved_register(*r as u8).expect("Out of saved registers"))
                                        .unwrap_or(register::index("s0").unwrap());
                                    current_saved.push(next_saved_reg as i64);

                                    self.symtab.update(".FUNC_SAVED_REGISTERS", Symbol::new(Value::Integers(current_saved)));

                                    self.symtab.insert(arg, Symbol::new(Value::Register(next_saved_reg)));

                                    return Ok(0);
                                } else if value == ".temp" {
                                    if exprs.len() != 2 {
                                        return Err(AsmProcessError::DirectiveFormat);
                                    }

                                    let arg = exprs[1].get_identifier().ok_or(AsmProcessError::DirectiveFormat)?;

                                    let mut current_temp = self.symtab.get_immediate(".FUNC_TEMP_REGISTERS").unwrap().value.get_integers().clone();
                                    let next_temp_reg = current_temp.last()
                                        .map(|r| register::next_temp_register(*r as u8).expect("Out of temp registers"))
                                        .unwrap_or(register::index("t0").unwrap());
                                    current_temp.push(next_temp_reg as i64);

                                    self.symtab.update(".FUNC_TEMP_REGISTERS", Symbol::new(Value::Integers(current_temp)));

                                    self.symtab.insert(arg, Symbol::new(Value::Register(next_temp_reg)));

                                    return Ok(0);
                                } else if value == ".stack" {
                                    if exprs.len() != 3 {
                                        return Err(AsmProcessError::DirectiveFormat);
                                    }

                                    let current_stack_alloc = self.symtab.get_immediate(".FUNC_STACK_ALLOCATED").unwrap().value.eval();

                                    let offset_id = exprs[1].get_identifier().ok_or(AsmProcessError::DirectiveFormat)?;
                                    let size_in_bytes = calc::calculate(&exprs[2], self.symtab).map_err(|e| AsmProcessError::CalcError(format!("{:?}", e)))?;

                                    // word alignment...
                                    let alloc = (size_in_bytes + 3) / 4 * 4;

                                    let next_stack_alloc = current_stack_alloc + alloc;

                                    self.symtab.insert(offset_id, Symbol::new(Value::General(current_stack_alloc)));
                                    self.symtab.insert(&format!("{}.LEN", offset_id), Symbol::new(Value::General(size_in_bytes)));

                                    self.symtab.update(".FUNC_STACK_ALLOCATED", Symbol::new(Value::General(next_stack_alloc)));

                                    return Ok(0);
                                }
                            }
                            _ => ()
                        }
                    }
                }
                _ => ()
            }

            // End of preamble
            self.symtab.update(".FUNC_PREAMBLE", Symbol::new(Value::General(0)));

            inc += self.insert_func_prologue(counter + inc)?;
        }

        inc += self.handler.handle(expr, self.symtab, counter + inc)?;

        Ok(inc)
    }

    fn enter_scope(&mut self, scope_name: &Expr, file_name: &str, counter: u32) -> Result<u32, AsmProcessError> {
        println!("enter_scope with scope_name={:?}, file_name={}, counter:0x{:08X}", scope_name, file_name, counter);

        let mut inc = 0u32;

        // Finish previous function preamble
        inc += self.finish_func_preamble(counter + inc)?;

        let directive = scope_name.get_identifier().unwrap();

        if directive == ".begin" {
            // install new symtab...
            self.symtab.push_env((String::from(file_name), scope_name.token().clone()));

        } else if directive == ".block" {
            self.symtab.push_env((String::from(file_name), scope_name.token().clone()));

            /*
            internal symbol used - make sure to access from only immediate symtab node
            * .BLOCK_NAME : String = block name
            * .BLOCK_PREAMBLE : General = 1 for preamble

            symbols exposed
            * <block name>.BEGIN : location
            * <block name>.END : location

            */

            self.symtab.insert(".BLOCK_PREAMBLE", Symbol::new(Value::General(1)));

        } else if directive == ".func" || directive == ".leaf" {
            self.symtab.push_env((String::from(file_name), scope_name.token().clone()));

            /*
            Internal symbols used
            * .FUNC_PREAMBLE : General = 1 for preamble
            * .FUNC_SAVED_REGISTERS : Integers. List of saved registers that are evicted, not including sp and ra...
            * .FUNC_TEMP_REGISTERS : Integers. No use for this for now...
            * .FUNC_STACK_ALLOCATED : General for allocating stack

            * .FUNC_STACK_FRAME_SIZE : General

            symbols exposed
            * RETURN : location

            * .var symbols
            * .temp symbols
            * .stack offset symbols
            * <stack offset symbol>.LEN - length in bytes (not including padding)

            */

            self.symtab.insert(".FUNC_PREAMBLE", Symbol::new(Value::General(1)));
//            let saved_registers = vec![register::index("sp").unwrap() as i64, register::index("ra").unwrap() as i64];

            self.symtab.insert(".FUNC_SAVED_REGISTERS", Symbol::new(Value::Integers(Vec::new())));
            self.symtab.insert(".FUNC_TEMP_REGISTERS", Symbol::new(Value::Integers(Vec::new())));
            self.symtab.insert(".FUNC_STACK_ALLOCATED", Symbol::new(Value::General(0)));

        } else {
            return Err(AsmProcessError::UnknownDirective(String::from(directive)))
        }

        Ok(inc)
    }

    fn exit_scope(&mut self, counter: u32) -> Result<u32, AsmProcessError> {
        println!("exit_scope with counter=0x{:08X}", counter);
        self.symtab.print();

        let mut inc = 0u32;

        let block_name = match self.symtab.get_immediate(".BLOCK_NAME") {
            Some(s) => Some(s.value.get_string().clone()),
            None => None
        };

        if let Some(s) = block_name {
            self.symtab.insert(&(s + ".END"), Symbol::new(Value::Location(counter)));
        }

        inc += self.finish_func_preamble(counter)?;

        match self.symtab.get_immediate(".FUNC_PREAMBLE") {
            Some(_) => {
                self.symtab.insert("RETURN", Symbol::new(Value::Location(counter)));

                //TODO inject function epilogue
                /*
                (addi sp sp stack_frame_size)
                (lw s0 sp -8)
                (lw ra sp -4)
                (jalr zero ra 0)
                */

                let stack_frame_size = self.symtab.get_immediate(".FUNC_STACK_FRAME_SIZE").unwrap().value.eval();
                inc += self.handler.handle(&parser::parse(&format!("(addi sp sp {})", stack_frame_size)), self.symtab, counter + inc).unwrap();

                let saved_registers = self.symtab.get_immediate(".FUNC_SAVED_REGISTERS").unwrap().value.get_integers().clone();

                for i in (0..saved_registers.len()).rev() {
                    let reg = saved_registers[i];
                    let offset = (i as i32 + 2) * -4;
                    let line = format!("(lw {} sp {})", reg, offset);
                    inc += self.handler.handle(&parser::parse(&line), self.symtab, counter + inc).unwrap();
                }

                inc += self.handler.handle(&parser::parse("(lw ra sp -4)"), self.symtab, counter + inc).unwrap();
                inc += self.handler.handle(&parser::parse("(jalr zero ra 0)"), self.symtab, counter + inc).unwrap();

            }
            None => (),
        }

        self.symtab.pop_env();
        Ok(inc)
    }

    fn org(&mut self, counter: u32) -> Result<(), AsmProcessError> {
        // ...
        self.handler.org(counter)
    }
}