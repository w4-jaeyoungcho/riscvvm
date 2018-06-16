mod test;

use std;
use ::lexer::*;

pub struct Parser<R: std::io::Read> {
    lexer: Lexer<R>,
}

impl<R: std::io::Read> Parser<R> {
    pub fn new(reader: R) -> Parser<R> {
        Parser {
            lexer: Lexer::new(reader),
        }
    }

    pub fn next(&mut self) -> Option<Result<Expr, ParserError>> {
        let token = match self.lexer.next() {
            Ok(token) => token,
            Err(LexerError { error: LexerErrorVariant::EOF, .. }) => return None,
            Err(e) => return Some(Err(ParserError::Lower(e))),
        };

        Some(self.process_token(token))
    }

    // Read one expression
    fn process_token(&mut self, token: Token) -> Result<Expr, ParserError> {
        match token.lexeme.clone() {
            Lexeme::SyntaxOpeningParen => {
                self.process_list(token)
            }
            Lexeme::SyntaxClosingParen => {
                Err(ParserError::UnexpectedClosingParen(token))
            }
            Lexeme::String(s) => Ok(Expr::String { value: s, token: token}),
            Lexeme::Identifier(s) => {
                Ok(Expr::Identifier { value: s, token: token })
            }
            Lexeme::Integer(i) => {
                Ok(Expr::Integer { value: i, token: token })
            }
            other => panic!("Not yet implemented: {:?}", &other)
        }
    }

    fn process_list(&mut self, opening_token: Token) -> Result<Expr, ParserError> {
        // read and process until closing paren
        assert!(opening_token.lexeme == Lexeme::SyntaxOpeningParen);

        let mut exprs = Vec::new();

        let closing_token = loop {
            let token = match self.lexer.next() {
                Ok(token) => token,
                Err(LexerError { error: LexerErrorVariant::EOF, .. }) => return Err(ParserError::UnexpectedEOF(opening_token)),
                Err(e) => return Err(ParserError::Lower(e)),
            };

            match token.lexeme.clone() {
                Lexeme::SyntaxClosingParen => break token.clone(), // consume
                _ => {
                    let e = self.process_token(token)?;
                    exprs.push(e);
                }
            }
        };

        Ok(Expr::List { exprs: exprs, opening_token: opening_token, closing_token: closing_token })
    }
}

#[derive(Debug)]
pub enum Expr {
    Identifier { value: String, token: Token },
    Integer { value: i64, token: Token },
    String { value: String, token: Token },
    List { exprs: Vec<Expr>, opening_token: Token, closing_token: Token },
}

impl Expr {
    pub fn token<'a>(&'a self) -> &'a Token {
        match self {
            &Expr::Identifier { token: ref token, .. } => token,
            &Expr::Integer { token: ref token, .. } => token,
            &Expr::String { token: ref token, .. } => token,
            &Expr::List { opening_token: ref token, .. } => token,
        }
    }

    pub fn get_identifier<'a>(&'a self) -> Option<&'a str> {
        match self {
            &Expr::Identifier { value: ref s, .. } => Some(&s),
            _ => None,
        }
    }

    pub fn get_integer(&self) -> Option<i64> {
        match self {
            &Expr::Integer { value: v, .. } => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ParserError {
    UnexpectedEOF(Token),
    // Opening paren
    UnexpectedClosingParen(Token),
    Lower(LexerError),
}

impl ParserError {
    pub fn token<'a>(&'a self) -> Option<&'a Token> {
        match self {
            &ParserError::UnexpectedEOF(ref token) => Some(token),
            &ParserError::UnexpectedClosingParen(ref token) => Some(token),
            &ParserError::Lower(_) => None,
        }
    }

    // (line_index, column_index)
    pub fn pos(&self) -> (usize, usize) {
        match self {
            &ParserError::UnexpectedEOF(ref token) => (token.line_index, token.column_index),
            &ParserError::UnexpectedClosingParen(ref token) => (token.line_index, token.column_index),
            &ParserError::Lower(ref error) => (error.line_index, error.column_index),
        }
    }
}

use ::lexer::mem_reader;

// Parses one Expr from string
pub fn parse(s: &str) -> Expr {
    let mut parser = Parser::new(mem_reader::MemReader::new(s.as_bytes()));
    let expr = parser.next().expect("parser next").expect("parser next result");
    expr
}