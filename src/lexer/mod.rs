// Read Scheme-like input stream - nah, just

mod test;
pub mod mem_reader;
mod read_unreader;

use std;
use std::io::Read;

use self::read_unreader::*;

#[derive(Debug)]
pub enum LexerErrorVariant {
    EOF,
    IntFormat { literal: String },
    StringLiteralUnexpectedEOF { literal: String },
    StringLiteralUnhandledEscapeSequence { literal: String },
    Lower(std::io::Error),
}

#[derive(Debug)]
pub struct LexerError {
    pub line_index: usize,
    pub column_index: usize,
    pub error: LexerErrorVariant,
}

impl LexerError {
    pub fn is_eof(&self) -> bool {
        match &self.error {
            &LexerErrorVariant::EOF => true,
            _ => false,
        }
    }
}

pub struct Lexer<R: Read> {
    lower: ReadUnreader<R>,

    not_start_of_line: bool,

    line_index: usize,
    column_index: usize,

    token_start_pos: usize,
    token_line_index: usize,
    token_column_index: usize,
    token_literal: String,
}

impl<R: Read> Lexer<R> {
    pub fn new(read: R) -> Lexer<R> {
        return Lexer {
            lower: ReadUnreader::new(read),

            not_start_of_line: false,

            line_index: 0,
            column_index: 0,

            token_start_pos: 0,
            token_line_index: 0,
            token_column_index: 0 ,
            token_literal: String::new(),
        };
    }

    fn new_error(&self, error: LexerErrorVariant) -> LexerError {
        LexerError {
            line_index: self.line_index,
            column_index: self.column_index,
            error: error,
        }
    }

    fn read_lower(&mut self) -> Result<char, LexerErrorVariant> {
        self.lower.read().map_err(|e| {
            match e {
                ReadUnreaderError::EOF => LexerErrorVariant::EOF,
                ReadUnreaderError::Lower(e) => LexerErrorVariant::Lower(e),
            }
        })
    }

    fn reset_token(&mut self) {
        self.token_start_pos = self.lower.pos;
        self.token_line_index = self.line_index;
        self.token_column_index = self.column_index;
        self.token_literal.clear();
    }

    fn read(&mut self) -> Result<char, LexerErrorVariant> {
        let c = self.read_lower()?;

        if c == '\n' {
            self.not_start_of_line = false;
            self.line_index += 1;
            self.column_index = 0;
        } else {
            self.not_start_of_line = true;
            self.column_index += 1;
        }

        self.token_literal.push(c);

        Ok(c)
    }

    fn unread(&mut self, num: usize) {
        for _ in 0..num {
            let c = self.lower.unread();

            if c == '\n' {
                self.line_index -= 1;
                // column_index is lost
            }

            self.token_literal.pop();
        }
    }

    fn peek(&mut self) -> Result<char, LexerErrorVariant> {
        self.lower.peek().map_err(|e| {
            match e {
                ReadUnreaderError::EOF => LexerErrorVariant::EOF,
                ReadUnreaderError::Lower(e) => LexerErrorVariant::Lower(e),
            }
        })
    }

    fn eat_through_whitespace(&mut self) -> Result<(), LexerErrorVariant> {
        loop {
            let prime = self.peek()?;

            if !prime.is_whitespace() && prime != '\n' {
                // Not whitespace

//                if !self.not_start_of_line && prime == ';' {
                if prime == ';' {
                    // comment line
                    // consume up to newline
                    self.read()?; // consume ';'

                    loop {
                        let cur = self.peek()?;

                        if cur == '\n' {
                            self.read()?;
                            break
                        }

                        // consume
                        self.read();
                    }

                    continue
                } else {
                    // Done
                    // clear token to start from prime
                    self.reset_token();
                    return Ok(());
                }
            }

            // consume that whitespace
            self.read()?;
        }
    }

    fn next_lower(&mut self) -> Result<Token, LexerErrorVariant> {
        // Eat through whitespace and comment
        self.eat_through_whitespace()?;

        let cur = self.read()?;
        assert!(!cur.is_whitespace() && cur != '\n');

        // Got one

        // Syntaxes
        if cur == '(' {
            return Ok(self.new_token(Lexeme::SyntaxOpeningParen));
        } else if cur == ')' {
            return Ok(self.new_token(Lexeme::SyntaxClosingParen));
        }

        // integers
        if cur.is_digit(10) || cur == '-' {
            // This is a number - read till non-number
            self.read_token()?;

            let literal = self.token_literal.clone();

            if literal == "-" {
                // identifier '-'
                let token = self.new_token(Lexeme::Identifier(literal));
                return Ok(token);

            } else {
                //            let value = i64::from_str(&literal).map_err(|_| LexerErrorVariant::IntFormat {literal: literal})?;
                // Rust is no help. I have to handle hexadecimal number format (0x1234)
                let value = parse_number(&literal).map_err(|_| LexerErrorVariant::IntFormat { literal: literal })?;

                let token = self.new_token(Lexeme::Integer(value));

                return Ok(token);
            }
        }

        // string literal
        if cur == '"' {
            // Read until next ", screw escapes for now
            let mut s = String::new();
            loop {
                let c = self.read().map_err(|e| {
                    match &e {
                        &LexerErrorVariant::EOF => LexerErrorVariant::StringLiteralUnexpectedEOF { literal: self.token_literal.clone() },
                        _ => e,
                    }
                })?;

                if c == '"' {
                    break
                }

                // a char value appended
                s.push(c);
            }
            let token = self.new_token(Lexeme::String(s));
            return Ok(token);
        }

        // character

        // identifier
        {
            self.read_token()?;

            let literal = self.token_literal.clone();

            let token = self.new_token(Lexeme::Identifier(literal));
            return Ok(token);
        }

        // ?

//        Err(LexerErrorVariant::EOF)
    }

    pub fn next(&mut self) -> Result<Token, LexerError> {
        self.next_lower().map_err(|e| self.new_error(e))
    }

    // read up to syntaxes or whitespace
    fn read_token(&mut self) -> Result<(), LexerErrorVariant> {
        loop {
            let curr = match self.peek() {
                Ok(c) => c,
                Err(LexerErrorVariant::EOF) => break,
                Err(e) => return Err(e),
            };

            // Check if end of token
            if curr.is_whitespace() {
                break
            }

            if curr == '\n' {
                break
            }

            if curr == '(' || curr == ')' {
                break
            }

            // Consume that
            self.read()?;
        }

        Ok(())
    }

    fn new_token(&mut self, lexeme: Lexeme) -> Token {
        let token = Token {
            lexeme: lexeme,
            literal: std::mem::replace(&mut self.token_literal, String::new()),
            // LOL
            pos: self.token_start_pos,
            line_index: self.token_line_index,
            column_index: self.token_column_index,
        };

        self.reset_token();

        token
    }
}

fn parse_number(literal: &str) -> Result<i64, std::num::ParseIntError> {
    assert!(literal.len() >= 0);
    let chars: Vec<char> = literal.chars().collect();
    if chars.len() >= 2 && &chars[0..2] == &['0', 'x'] {
        // hexadecimal
//        let s: String = (&chars[2..]).collect();
        let s = &literal[2..];
        if s.len() == 0 {
            return Ok(0);
        } else {
            i64::from_str_radix(&literal[2..], 16)
        }
    } else {
        // decimal
        i64::from_str_radix(literal, 10)
    }
}

// Lexeme
#[derive(PartialEq, Eq)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(Hash)]
pub enum Lexeme {
    SyntaxOpeningParen,
    SyntaxClosingParen,
    Identifier(String),
    Integer(i64),
    Character(char),
    String(String)
}

impl Lexeme {
    pub fn placeholder() -> Lexeme {
        Lexeme::String(String::from("vasm placeholder"))
    }
}

#[derive(PartialEq, Eq)]
#[derive(Clone)]
#[derive(Debug)]
#[derive(Hash)]
pub struct Token {
    pub lexeme: Lexeme,

    pub literal: String,
    pub pos: usize,
    pub line_index: usize,
    pub column_index: usize,
}

impl Token {
    pub fn placeholder() -> Token {
        Token {
            lexeme: Lexeme::placeholder(),
            literal: String::from("placeholder"),
            pos: 0,
            line_index: 0,
            column_index: 0,
        }
    }
}