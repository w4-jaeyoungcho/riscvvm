#![cfg(test)]

use super::*;
use super::mem_reader::*;

fn test_lexer() {
    let sample_text = String::from("Bananas (are 2 \"very\") ()");

    let reader = MemReader::new(sample_text.as_bytes());

    let mut lexer = Lexer::new(reader);

    match lexer.next() {
        Ok(token) => {
            let expected = Token {
                lexeme: Lexeme::Identifier(String::from("Bananas")),
                literal: String::from("Bananas"),
                pos: 0,
                line_index: 0,
                column_index: 0,
            };
            assert_eq!(token, expected);
        }
        Err(e) => panic!("{:?}", e),
    }
}

#[test]
fn test_syntaxes() {
    let sample_text = "(23)\nwhat(\"bananas are deli\"";

    let reader = MemReader::new(sample_text.as_bytes());

    let mut lexer = Lexer::new(reader);

    match lexer.next() {
        Ok(token) => {
            let expected = Token {
                lexeme: Lexeme::SyntaxOpeningParen,
                literal: String::from("("),
                pos: 0,
                line_index: 0,
                column_index: 0,
            };
            assert_eq!(token, expected);
        }
        Err(e) => panic!("{:?}", e),
    }

    {
        let token = lexer.next().unwrap();
        let expected = Token {
            lexeme: Lexeme::Integer(23),
            literal: String::from("23"),
            pos: 1,
            line_index: 0,
            column_index: 1,
        };
        assert_eq!(token, expected);
    }

    match lexer.next() {
        Ok(token) => {
            let expected = Token {
                lexeme: Lexeme::SyntaxClosingParen,
                literal: String::from(")"),
                pos: 3,
                line_index: 0,
                column_index: 3,
            };
            assert_eq!(token, expected);
        }
        Err(e) => panic!("{:?}", e),
    }

    {
        let token = lexer.next().unwrap();
        let value = String::from("what");
        let expected = Token {
            lexeme: Lexeme::Identifier(value.clone()),
            literal: value,
            pos: 5,
            line_index: 1,
            column_index: 0,
        };

        assert_eq!(token, expected);
    }

    match lexer.next() {
        Ok(token) => {
            let expected = Token {
                lexeme: Lexeme::SyntaxOpeningParen,
                literal: String::from("("),
                pos: 9,
                line_index: 1,
                column_index: 4,
            };
            assert_eq!(token, expected);
        }
        Err(e) => panic!("{:?}", e),
    }

    match lexer.next() {
        Ok(token) => {
            let expected = Token {
                lexeme: Lexeme::String(String::from("bananas are deli")),
                literal: String::from("\"bananas are deli\""),
                pos: 10,
                line_index: 1,
                column_index: 5,
            };
            assert_eq!(token, expected);
        }
        Err(e) => panic!("{:?}", e),
    }

    match lexer.next() {
        Ok(token) => {
            panic!("What: {:?}", token);
        }
        Err(LexerError{ error: LexerErrorVariant::EOF, .. }) => {
            // good
        }
        Err(e) => {
            panic!("Seriously {:?}", e);
        }
    }
}


#[test]
fn test_minus_sign() {
    let sample_text = "-23 - banana";

    let reader = MemReader::new(sample_text.as_bytes());

    let mut lexer = Lexer::new(reader);

    match lexer.next() {
        Ok(token) => {
            let expected = Token {
                lexeme: Lexeme::Integer(-23),
                literal: String::from("-23"),
                pos: 0,
                line_index: 0,
                column_index: 0,
            };
            assert_eq!(token, expected);
        }
        Err(e) => panic!("{:?}", e),
    }

    match lexer.next() {
        Ok(token) => {
            let expected = Token {
                lexeme: Lexeme::Identifier(String::from("-")),
                literal: String::from("-"),
                pos: 4,
                line_index: 0,
                column_index: 4,
            };
            assert_eq!(token, expected);
        }
        Err(e) => panic!("{:?}", e),
    }

}