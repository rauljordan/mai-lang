use std::iter::Peekable;
use std::ops::DerefMut;
use std::str::Chars;

use thiserror::Error;
use eyre::Result;

use crate::token::Token;

#[derive(Debug,Error)]
pub enum LexingError {
    #[error("unknown token matched `{0}`")]
    UnknownToken(String),
}

pub type LexResult = Result<Token, LexingError>;

pub struct TokenLexer<'a> {
    input: &'a str,
    chars: Box<Peekable<Chars<'a>>>,
    curr: usize,
}

impl<'a> Iterator for TokenLexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        match self.lex() {
            Ok(Token::EOF) => None,
            Err(_) => None,
            Ok(token) => Some(token),
        }
    }
}

impl<'a> TokenLexer<'a> {
    pub fn new(input: &'a str) -> TokenLexer<'a> {
        TokenLexer {
            input,
            chars: Box::new(input.chars().peekable()),
            curr: 0,
        }
    }

    pub fn lex(&mut self) -> LexResult {
        let chars = self.chars.deref_mut();
        let src = self.input;

        let mut curr = self.curr;
        loop {
            {
                let ch = chars.peek();
                if ch.is_none() {
                    self.curr = curr;
                    return Ok(Token::EOF);
                }
                if !ch.unwrap().is_whitespace() {
                    break;
                }
            }
            chars.next();
            curr += 1;
        }

        let start = curr;
        let next = chars.next();

        if next.is_none() {
            return Ok(Token::EOF);
        }

        curr += 1;

        let result = match next.unwrap() {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            '0'..='9' | '.' => {
                loop {
                    let ch = match chars.peek() {
                        Some(ch) => *ch,
                        None => return Ok(Token::EOF),
                    };
                    if ch != '.' && !ch.is_ascii_hexdigit() {
                        break;
                    }
                    chars.next();
                    curr += 1;
                }
                Ok(Token::Number(src[start..curr].parse().unwrap()))
            },

            'a'..='z' | 'A'..='Z' | '_' => {
                loop {
                    let ch = match chars.peek() {
                        Some(ch) => *ch,
                        None => return Ok(Token::EOF),
                    };
                    if ch != '_' && !ch.is_alphanumeric() {
                        break;
                    }
                    chars.next();
                    curr += 1;
                }

                match &src[start..curr] {
                    "var" => Ok(Token::Var),
                    "if" => Ok(Token::If),
                    "else" => Ok(Token::Else),
                    "false" => Ok(Token::False),
                    "true" => Ok(Token::True),
                    "wagmi" => Ok(Token::Wagmi),
                    ident => Ok(Token::Ident(ident.to_string())),
                }
            },
            op => {
                // Peeks to see if the next character is
                // is the one specified, and returns the token at the first
                // argument, otherwise returns the last argument.
                // Useful to check if we receive an operation such as `=` and want to
                // check if the next character is `=` to return Token::Eqq otherwise Token::Eq
                macro_rules! peek_next_otherwise {
                    ($char:expr, $require:expr,$otherwise:expr) => {
                        match chars.peek() {
                            Some($char) => {
                                chars.next();
                                curr += 1;
                                Ok($require)
                            }
                            _ => Ok($otherwise),
                        }
                    };
                }
                match op {
                    '+' => Ok(Token::Plus),
                    '-' => Ok(Token::Minus),
                    '*' => Ok(Token::Times),
                    '/' => Ok(Token::Div),
                    '!' => peek_next_otherwise!('=', Token::Neq, Token::Bang),
                    '=' => peek_next_otherwise!('=', Token::Eqq, Token::Eq),
                    '<' => peek_next_otherwise!('=', Token::Leq, Token::Less),
                    '>' => peek_next_otherwise!('=', Token::Geq, Token::Greater),
                    unknown => Err(LexingError::UnknownToken(unknown.to_string()))
                }
            },
        };
        self.curr = curr;
        result
    }
}
