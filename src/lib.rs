
extern crate backtrace;
#[macro_use]
extern crate log;
#[macro_use] extern crate enum_primitive;
extern crate num;

mod lexer;
mod parser;
mod arch;
mod translate;
pub mod asm;
pub mod disasm;
mod encode;
mod decode;
pub mod machine;
//pub mod vpc;
mod calc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
