// rlox: Lox interpreter/compiler in Rust.
//    Copyright 2023 James Smith <j@mes.sh>
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

pub mod ast;

use crate::errors::{ParseError, Result, RloxError};
use crate::tokens::{Token, TokenLiteral, TokenType};
use ast::{Expr, ExprLiteral, Stmt};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // Build up the AST by precendence
    // Parser methods
    pub fn parse(&mut self) -> Vec<Result<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            // parse the next declaration
            let stmt = self.declaration();
            if stmt.is_err() {
                // if there's an error, synchronize to the next synchronization point
                self.synchronize();
            }
            statements.push(stmt);
        }
        statements
    }

    // Check for variable declarations
    fn declaration(&mut self) -> Result<Stmt> {
        if self.is_any_tokens(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    // Handle variable declaration
    fn var_declaration(&mut self) -> Result<Stmt> {
        let token = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .token_literal()
            .to_string();

        let initializer = match self.is_any_tokens(&[TokenType::Equal]) {
            true => Some(self.expression()?),
            false => None,
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;

        Ok(Stmt::Var(token, initializer))
    }

    // Statement functions

    fn statement(&mut self) -> Result<Stmt> {
        if self.is_any_tokens(&[TokenType::If]) {
            // If condition
            self.if_stmt()
        } else if self.is_any_tokens(&[TokenType::Print]) {
            // We're a `print` statement
            self.print_stmt()
        } else if self.is_any_tokens(&[TokenType::LeftBrace]) {
            // New block/scope
            self.block_stmt()
        } else if self.is_any_tokens(&[TokenType::While]) {
            // While loop
            self.while_stmt()
        } else if self.is_any_tokens(&[TokenType::For]) {
            // For loop
            self.for_stmt()
        } else {
            // We're an expression
            self.expression_stmt()
        }
    }

    // Evaluate the expression and return Stmt::Print(result)
    fn print_stmt(&mut self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn block_stmt(&mut self) -> Result<Stmt> {
        let mut stmts = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(Stmt::Block(stmts))
    }

    fn if_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let mut else_branch: Option<Box<Stmt>> = None;

        if self.is_any_tokens(&[TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If(condition, Box::new(then_branch), else_branch))
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;

        let stmts = self.statement()?;

        Ok(Stmt::While(condition, Box::new(stmts)))
    }

    // For loops are de-sugared into a while loop with optional initializer
    fn for_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        // Handle optional initializer
        let initializer = match self.is_any_tokens(&[TokenType::Semicolon]) {
            true => None, // no initializer
            false => match self.is_any_tokens(&[TokenType::Var]) {
                true => Some(self.var_declaration()?), // new variable declaration
                false => Some(self.expression_stmt()?), // expression
            },
        };

        // Handle optional condition before each loop
        let condition = match self.is_any_tokens(&[TokenType::Semicolon]) {
            true => None,
            false => Some(self.expression()?),
        };

        self.consume(TokenType::Semicolon, "Expect ';' after condition.")?;

        // Handle optional increment expression after each loop
        let increment = match self.is_any_tokens(&[TokenType::Semicolon]) {
            true => None,
            false => Some(self.expression()?),
        };

        self.consume(TokenType::RightParen, "Expect ')' after increment.")?;

        // We start de-sugaring the for() by appending the increment expression to the end
        // of the body. Then we'll transform the body into the body of a while loop and
        // attach its condition. Finally, we prepend it with the initializer.
        let mut body = self.statement()?;

        // if there's an increment, append it to the body
        if let Some(expr) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(expr)]);
        }

        match condition {
            None => {
                // Make the condition `true` and create a while (true) loop
                body = Stmt::While(Expr::Literal(ExprLiteral::Bool(true)), Box::new(body));
            }
            Some(expr) => {
                // Otherwise, create a while (condition) loop
                body = Stmt::While(expr, Box::new(body));
            }
        }

        // Prepend the initializer if it exists
        if let Some(expr) = initializer {
            body = Stmt::Block(vec![expr, body]);
        }

        Ok(body)
    }

    // Evaluate the expression and return Stmt::Expression(result)
    fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Stmt::Expression(expr))
    }

    // Expression functions
    // Descend!
    // Assignment > OR > AND > equality > comparison
    // > term > factor > unary > primary

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.is_any_tokens(&[TokenType::Equal]) {
            let value = self.assignment()?;

            return if let Expr::Variable(name) = expr {
                Ok(Expr::Assign(name, Box::new(value)))
            } else {
                Err(RloxError::Parse(ParseError::ParseFailure(
                    "Invalid assignment target.".to_string(),
                )))
            };
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.is_any_tokens(&[TokenType::Or]) {
            let operator = self.previous().token_type();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.is_any_tokens(&[TokenType::And]) {
            let operator = self.previous().token_type();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        // Loop over equality expression, building up the AST with recursive Binary Expressions
        // a == b == c == d == e != f ...
        while self.is_any_tokens(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().token_type(); // one of ==, !=
            let rhs = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.is_any_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().token_type(); // one of >, >=, <, <+
            let rhs = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.is_any_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().token_type(); // one of +, -
            let rhs = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.is_any_tokens(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().token_type();
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.is_any_tokens(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous().token_type();
            let rhs = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(rhs)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.is_any_tokens(&[TokenType::False]) {
            return Ok(Expr::Literal(ExprLiteral::Bool(false)));
        }
        if self.is_any_tokens(&[TokenType::True]) {
            return Ok(Expr::Literal(ExprLiteral::Bool(true)));
        }
        if self.is_any_tokens(&[TokenType::Nil]) {
            return Ok(Expr::Literal(ExprLiteral::Nil));
        }

        if self.is_any_tokens(&[TokenType::Number, TokenType::String]) {
            match self.previous().token_literal() {
                TokenLiteral::Number(value) => {
                    return Ok(Expr::Literal(ExprLiteral::Number(*value)));
                }
                TokenLiteral::Str(value) => {
                    return Ok(Expr::Literal(ExprLiteral::String(value.clone())));
                }
                _ => {}
            }
        }

        if self.is_any_tokens(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;

            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.is_any_tokens(&[TokenType::Identifier]) {
            if let TokenLiteral::Identifier(literal) = self.previous().token_literal().clone() {
                return Ok(Expr::Variable(literal));
            } else {
                // Should not be reachable unless by programmer error in scanner.
                panic!("Expected identifier");
            }
        }

        if self.is_any_tokens(&[TokenType::Eof]) {
            return Err(RloxError::Parse(ParseError::EOF));
        }

        // TODO: Try to be smarter here.
        Err(RloxError::Parse(ParseError::UnexpectedToken(
            self.peek().to_string(),
        )))
    }

    // Helper functions

    fn is_any_tokens(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(*token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<&Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(RloxError::Parse(ParseError::UnexpectedToken(
            msg.to_string(),
        )))
    }

    fn is_at_end(&self) -> bool {
        self.check(TokenType::Eof)
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.peek().token_type() == token_type
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            // Continue scanning forward until we reach a SemiColon
            // or a keyword. This lets us get past a syntax error and
            // continue parsing.
            if self.previous().token_type() == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::*;

    fn get_scanner(opt: Option<&str>) -> crate::scanner::Scanner {
        let mut s = match opt {
            Some(x) => crate::scanner::Scanner::new(x),
            None => crate::scanner::Scanner::new("10 == 20;"),
        };
        s.scan_tokens().unwrap();
        s
    }

    fn get_parser_scanner(opt: Option<&str>) -> Parser {
        get_scanner(opt).into_parser()
    }

    #[test]
    fn basic_seeking() {
        let mut p = get_parser_scanner(None);

        assert!(p.check(TokenType::Number));

        assert_eq!(p.advance().token_type(), TokenType::Number);
        assert_eq!(p.advance().token_type(), TokenType::EqualEqual);
        assert_eq!(p.advance().token_type(), TokenType::Number);
        assert_eq!(p.advance().token_type(), TokenType::Semicolon);

        assert_eq!(p.peek().token_type(), TokenType::Eof);
    }

    #[test]
    fn seek_past_end() {
        let mut p = get_parser_scanner(Some("var abc = 45; if (abc >= 20) { return false };"));
        while !p.is_at_end() {
            p.advance();
        }

        // we shouldn't seek past the EOF token.
        p.advance();
        assert_eq!(p.peek().token_type(), TokenType::Eof);
        p.advance();
        assert_eq!(p.peek().token_type(), TokenType::Eof);
    }

    #[test]
    fn basic_matching_not_equal() {
        let mut p = get_parser_scanner(None);
        assert!(!p.is_any_tokens(&[TokenType::And, TokenType::Equal, TokenType::BangEqual]));
    }

    #[test]
    fn basic_matching_equal() {
        let mut p = get_parser_scanner(None);
        assert!(p.check(TokenType::Number));
        p.advance();
        assert!(p.is_any_tokens(&[TokenType::EqualEqual, TokenType::BangEqual]));
    }

    // #[test]
    // fn parsing_equality_bool() {
    //     let mut p = get_parser_scanner(Some("10 == 20;"));
    //     assert_eq!(p.expression(), Expr::Binary((), (), ()));
    //     todo!();
    // }
}
