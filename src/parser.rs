use crate::token::Token;

#[derive(Debug)]
pub enum Expr {
    BinaryExpr {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryExpr {
        op: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: String,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Box<Expr>>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Box<Expr>),
    Print(Box<Expr>),
}

#[derive(Debug)]
pub struct Parser {
    pub tokens: Vec<Token>,
    current: usize,
}


macro_rules! bin_expr {
    ($exp1: expr, $op: expr, $exp2: expr) => {
       Expr::BinaryExpr { left: Box::new($exp1), op: $op, right: Box::new($exp2) } 
    };
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    pub fn parse(&mut self) -> Vec<Box<Stmt>> {
        let mut statements = vec!();
        while !self.is_at_end() {
            statements.push(self.statement());
        }
        return statements;
    }
    pub fn statement(&mut self) -> Box<Stmt> {
        let expr = self.expressionStatement();
        Box::new(expr)
    }
    pub fn expressionStatement(&mut self) -> Stmt {
        let value = self.expression();
        self.consume(Token::Semicolon);
        Stmt::Expr(Box::new(value))
    }
    pub fn expression(&mut self) -> Expr {
        return self.equality();
    }
    pub fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        while self.check_match(vec!(
            Token::Eqq, 
            Token::BangEq,
        )) {
            let op = self.previous();
            let right = self.comparison();
            expr = bin_expr!(expr, op, right);
        }
        return expr;
    }
    pub fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        while self.check_match(vec!(
            Token::Greater, 
            Token::Geq, 
            Token::Less, 
            Token::Leq,
        )) {
            let op = self.previous();
            let right = self.term();
            expr = bin_expr!(expr, op, right);
        }
        return expr;
    }
    pub fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while self.check_match(
            vec!(Token::Minus, Token::Plus)
        ) {
            let op = self.previous();
            let right = self.factor();
            expr = bin_expr!(expr, op, right);
        }
        return expr;
    }
    pub fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while self.check_match(
            vec!(Token::Div, Token::Times)
        ) {
            let op = self.previous();
            let right = self.unary();
            expr = bin_expr!(expr, op, right);
        }
        return expr;
    }
    pub fn unary(&mut self) -> Expr {
        if self.check_match(
            vec!(Token::Bang, Token::Minus)
        ) {
            let op = self.previous();
            let right = self.unary();
            return Expr::UnaryExpr { op, right: Box::new(right) };
        }
        return self.call();
    }
    pub fn call(&mut self) -> Expr {
        let mut expr = self.primary();
        loop {
            if self.check_match(
                vec!(Token::LParen)
            ) {
                expr = self.finish_call(expr);
            } else {
                break;
            }
        }
        return expr;
    }
    pub fn finish_call(&mut self, expr: Expr) -> Expr {
        let mut args = vec!();
        if !self.check(Token::RParen) {
            let mut sub_expr = self.expression();
            args.push(Box::new(sub_expr));
            while self.check_match(
                vec!(Token::Comma)
            ) {
                sub_expr = self.expression();
                args.push(Box::new(sub_expr));
            }
        }
        let paren = self.consume(Token::RParen);
        return Expr::Call { callee: Box::new(expr), paren, args };
    }
    pub fn primary(&mut self) -> Expr {
        if self.check_match(vec!(Token::False)) {
            return Expr::Literal { value: "false".to_string() };
        }
        if self.check_match(vec!(Token::True)) {
            return Expr::Literal { value: "true".to_string() };
        }
        match self.peek() {
            Token::Number(n) => {
                self.advance();
                return Expr::Literal { value: n };
            },
            Token::Ident(id) => {
                self.advance();
                return Expr::Literal { value: id };
            },
            _ => {}
        }
        if self.check_match(vec!(Token::LParen)) {
            let expr = self.expression();
            self.consume(Token::RParen);
            return Expr::Grouping { expr: Box::new(expr) };
        }
        // TODO: Handle this edge case...
        return Expr::Literal { value: "false".to_string() };
    }
    fn check_match(&mut self, toks: Vec<Token>) -> bool {
        for tok in toks.iter() {
            if self.check(tok.clone()) {
                self.advance();
                return true;
            }
        }
        return false;
    }
    fn check(&self, tok: Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek() == tok;
    }
    fn consume(&mut self, tok: Token) -> Token {
        if self.check(tok) {
            self.advance();
        } 
        self.previous()
    }
    fn advance(&mut self) {
        self.current += 1;
        self.previous();
    }
    fn is_at_end(&self) -> bool {
        return self.peek() == Token::EOF
    }
    fn previous(&self) -> Token {
        if let Some(tok) = self.tokens.get(self.current-1) {
            return tok.clone();
        }
        return Token::EOF;
    }
    fn peek(&self) -> Token {
        if let Some(tok) = self.tokens.get(self.current) {
            return tok.clone();
        }
        return Token::EOF;
    }
}
