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
    Logical {
        op: Token,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: String,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        args: Vec<Box<Expr>>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Box<Stmt>>),
    Expr(Box<Expr>),
    Print(Box<Expr>),
    Return {
        keyword: Token,
        value: Option<Box<Expr>>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    If {
        cond: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Var {
        name: Token,
        initializer: Box<Expr>,
    },
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
            statements.push(self.declaration());
        }
        return statements;
    }
    pub fn declaration(&mut self) -> Box<Stmt> {
        if self.check_match(vec!(Token::Fun)) { 
            return self.function_declaration();
        }
        if self.check_match(vec!(Token::Var)) {
            return self.variable_declaration();
        }
        self.statement()
    }
    pub fn function_declaration(&mut self) -> Box<Stmt> {
        let name = self.consume_identifier();
        self.consume(Token::LParen);
        let mut params = vec![];
        if !self.check_match(vec!(Token::RParen)) {
            params.push(self.consume_identifier());
            while self.check_match(vec!(Token::Comma)) {
                params.push(self.consume_identifier());
            }
        }
        self.consume(Token::RParen);
        self.consume(Token::LBrace);
        let body = self.block();
        Box::new(Stmt::Function { name, params, body })
    }
    pub fn consume_identifier(&mut self) -> Token {
        match self.peek() {
            Token::Ident(_) => {
                self.advance();
                self.previous()
            },
            _ => panic!("cannot match")
        }
    }
    pub fn variable_declaration(&mut self) -> Box<Stmt> {
        let name = match self.peek() {
            Token::Ident(_) => {
                self.advance();
                self.previous()
            },
            _ => panic!("cannot match")
        };
        let mut initializer = Expr::Literal { value: "false".to_string() };
        if self.check_match(vec!(Token::Eq)) {
            initializer = self.expression();
        }
        self.consume(Token::Semicolon);
        Box::new(Stmt::Var{ name, initializer: Box::new(initializer) })
    }
    pub fn statement(&mut self) -> Box<Stmt> {
        if self.check_match(vec!(Token::For)) {
            return self.for_statement();
        }
        if self.check_match(vec!(Token::If)) {
            return Box::new(self.if_statement());
        }
        if self.check_match(vec!(Token::Return)) {
            return Box::new(self.return_statement());
        }
        if self.check_match(vec!(Token::While)) {
            return Box::new(self.while_statement());
        }
        if self.check_match(vec!(Token::LBrace)) {
            return Box::new(Stmt::Block(self.block()));
        }
        let expr = self.expression_statement();
        Box::new(expr)
    }
    pub fn for_statement(&mut self) -> Box<Stmt> {
        self.consume(Token::LParen);
        let initializer: Option<Box<Stmt>>;
        if self.check_match(vec!(Token::Semicolon)) {
            initializer = None;
        } else if self.check_match(vec!(Token::Var)) {
            initializer = Some(self.variable_declaration());
        } else {
            initializer = Some(Box::new(self.expression_statement()));
        }

        let mut cond: Option<Expr> = None;
        if !self.check_match(vec!(Token::Semicolon)) {
            cond = Some(self.expression());
        }
        self.consume(Token::Semicolon);

        let mut increment: Option<Expr> = None;
        if !self.check_match(vec!(Token::RParen)) {
            increment = Some(self.expression());
        }
        self.consume(Token::RParen);

        let mut body = self.statement();
        if increment.is_some() {
            let expr = Stmt::Expr(Box::new(increment.unwrap()));
            body = Box::new(Stmt::Block(vec![body, Box::new(expr)]));
        }

        if cond.is_some() {
            cond = Some(Expr::Literal { value: "true".to_string() });
        }

        body = Box::new(Stmt::While { condition: Box::new(cond.unwrap()), body });
        if initializer.is_some() {
            body = Box::new(Stmt::Block(vec![initializer.unwrap(), body]));
        }
        body
    }
    pub fn if_statement(&mut self) -> Stmt  {
        self.consume(Token::LParen);
        let cond = self.expression();
        self.consume(Token::RParen);
        let then_branch = self.statement();
        let mut else_branch = None;
        if self.check_match(vec!(Token::Else)) {
            else_branch = Some(self.statement());
        }
        Stmt::If { cond: Box::new(cond), then_branch, else_branch }
    }
    pub fn return_statement(&mut self) -> Stmt  {
        let keyword = self.previous();
        let mut value = None;
        if !self.check_match(vec!(Token::Semicolon)) {
            value = Some(Box::new(self.expression()));
        }
        self.consume(Token::Semicolon);
        Stmt::Return { keyword, value }
    }
    pub fn while_statement(&mut self) -> Stmt  {
        self.consume(Token::LParen);
        let cond = self.expression();
        self.consume(Token::RParen);
        let body = self.statement();
        return Stmt::While { condition: Box::new(cond), body }
    }
    pub fn block(&mut self) -> Vec<Box<Stmt>> {
        let mut statements = vec!();
        while !self.check_match(vec!(Token::RBrace)) && !self.is_at_end() {
            statements.push(self.declaration());
        }
        self.consume(Token::RBrace);
        return statements;
    }
    pub fn expression_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.consume(Token::Semicolon);
        Stmt::Expr(Box::new(value))
    }
    pub fn expression(&mut self) -> Expr {
        return self.assignment();
    }
    pub fn assignment(&mut self) -> Expr {
        let expr = self.or();
        if self.check_match(vec!(Token::Eq)) {
            let value = self.assignment();
            return match expr {
                Expr::Variable { name } => {
                    Expr::Assign { name, value: Box::new(value) } 
                },
                _ => panic!("invalid assignment"),
            }
        }
        return expr;
    }
    pub fn or(&mut self) -> Expr {
        let mut expr = self.and();
        while self.check_match(vec!(Token::Or)) {
            let op = self.previous();
            let right = self.and();
            expr = Expr::Logical { left: Box::new(expr), op, right: Box::new(right) }
        }
        return expr;
    }
    pub fn and(&mut self) -> Expr {
        let mut expr = self.equality();
        while self.check_match(vec!(Token::And)) {
            let op = self.previous();
            let right = self.equality();
            expr = Expr::Logical { left: Box::new(expr), op, right: Box::new(right) }
        }
        return expr;
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
            Token::Ident(_) => {
                self.advance();
                return Expr::Variable { name: self.previous() };
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
