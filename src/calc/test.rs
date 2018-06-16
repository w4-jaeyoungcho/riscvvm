
use super::*;

use ::translate::symtab::Symtab;
use ::parser::Parser;
use ::lexer::mem_reader::MemReader;

#[test]
fn test_calc() {
    let cases = [
        ("2", 2),
        ("-2", -2),
        ("(- 2)", -2),
        ("(+ (* 10 5) (/ 100 10) (- (+ 5 (% 7 2))))", 54),
        ("(& -1 0x1234)", 0x1234),
    ];

    let symtab = Symtab::prepopulated();

    for &(s, expected) in &cases {
        let result = calculate_str(s, &symtab);

        assert_eq!(result, expected, "result of {}", s);
    }
}

fn calculate_str(s: &str, symtab: &Symtab) -> i64 {

    let mut parser = Parser::new(MemReader::new(s.as_bytes()));

    let expr = parser.next().unwrap().unwrap();

    match calculate(&expr, symtab) {
        Ok(v) => v,
        Err(e) => panic!("{:?}", e),
    }
}
