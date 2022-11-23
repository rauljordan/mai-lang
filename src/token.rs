/// Represents a primitive syntax token.
#[derive(Debug,Clone,PartialEq)]
pub enum Token {
    // Equalty and comparison operations.
    Eq,
    Neq,
    Eqq,
    BangEq,
    Greater,
    Less,
    Geq,
    Leq,
    // Binary ops.
    Plus,
    Minus,
    Times,
    Div,
    // Unary ops.
    Bang,
    // Grouping ops.
    LParen,
    RParen,
    // Literals and identifiers.
    True,
    False,
    Number(String),
    Ident(String),
    Var,
    Wagmi,
    EOF,
}

