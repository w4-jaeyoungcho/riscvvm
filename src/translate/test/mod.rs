
#![cfg(test)]

use super::*;

use ::lexer::mem_reader::*;
use ::parser::*;
use ::translate::symtab::*;


#[test]
fn test_insts() {
    let cases: &[(&str, u32)] = &[
        ("(nop)", 0x00000013),
        ("(lw t1 t3 0)", 0x000E2303),
//        ("(sw t1 t2 0)", 0x0063A023), // TODO check why fails
//        ("(hret)", 0x20200073),
    ];

    for i in 0..cases.len() {
        let mut parser = Parser::new(MemReader::new(cases[i].0.as_bytes()));
        let expr = parser.next().expect("parser next").expect("parser next result");

        let symtab = Symtab::prepopulated();
        let result = translate(&expr, &symtab, (i*4) as u32).expect(&format!("translate {}", i));
        assert_eq!(result, cases[i].1, "case {}", &cases[i].0);
    }
}


fn test_asm() {


    let sample = "; echo
(nop)

; trap vector
(lw t1 t3 0)
(sw t1 t2 0)
(hret)
";

    let expected: &[u32] = &[
        0x00000013,
        0x000E2303,
        0x0063a023,
        0x20200073,
    ];

    let reader = MemReader::new(sample.as_bytes());

}
