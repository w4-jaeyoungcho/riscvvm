#![cfg(test)]

use super::*;

use ::lexer::*;
use ::lexer::mem_reader::*;


fn test_parser() {
    let mut parser = Parser::new(MemReader::new("(+ 1 banana)".as_bytes()));

    let expr = parser.next().unwrap().unwrap();

    let list = match &expr {
        &Expr::List{ ref exprs, .. } => exprs,
        other => panic!("{:?}", &other),
    };

    let op = &list[0];

    match op {
        &Expr::Identifier{ value: ref value, .. } => {
            if value != "+" {
                panic!("unexpected value {}", value);
            }
        }
        other => {
            panic!("unexpected op: {:?}", other);
        }
    }
}

#[test]
fn test_parse_identifier() {
    let mut parser = Parser::new(MemReader::new("banana 72".as_bytes()));

    {
        let expr = parser.next().unwrap().unwrap();

        let s = match &expr {
            &Expr::Identifier { value: ref s, .. } => s,
            other => panic!("{:?}", &other),
        };

        assert_eq!(s, "banana");
    }

    {
        match &parser.next().unwrap().unwrap() {
            &Expr::Integer { value: i, .. } => {
                assert_eq!(i, 72);
            }
            other => panic!("{:?}", other)
        }
    }
}

#[test]
fn test_list() {
    let mut parser = Parser::new(MemReader::new("(+ 1 (what)) 2".as_bytes()));

    {
        match &parser.next().unwrap().unwrap() {
            &Expr::List { exprs: ref exprs, .. } => {
                match &exprs[0] {
                    &Expr::Identifier { value: ref s, .. } => assert_eq!(s, "+"),
                    other => panic!("got {:?}", &other),
                }

                match &exprs[1] {
                    &Expr::Integer { value: 1, .. } => (),
                    other => panic!("got {:?}", &other),
                }

                match &exprs[2] {
                    &Expr::List { exprs: ref exprs, .. } => {
                        assert_eq!(exprs.len(), 1);
                    }
                    other => panic!("got {:?}", &other),
                }
            }
            other => panic!("{:?}", &other),
        };
    }

    {
        match &parser.next().unwrap().unwrap() {
            &Expr::Integer { value: i, .. } => {
                assert_eq!(i, 2);
            }
            other => panic!("{:?}", other)
        }
    }
}