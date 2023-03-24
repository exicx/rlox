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

pub(crate) mod ast {
    use crate::tokens::TokenType;

    #[derive(Debug)]
    pub enum ExprLiteral {
        True,
        False,
        Number(f64),
        String(String),
        Nil,
    }

    #[derive(Debug)]
    pub enum Expr {
        Grouping(Box<Expr>),
        Binary(Box<Expr>, TokenType, Box<Expr>),
        Unary(TokenType, Box<Expr>),
        Literal(ExprLiteral),
    }
}

use self::ast::{Expr, ExprLiteral};
use crate::errors::{ParseError, RloxError};
use crate::tokens::{Token, TokenLiteral, TokenType};

type Result<T> = std::result::Result<T, RloxError>;

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

    pub fn parse(&mut self) -> Option<Expr> {
        if let Ok(expr) = self.expression() {
            Some(expr)
        } else {
            None
        }
    }

    fn expression(&mut self) -> Result<Expr> {
        let expr = self.equality();
        if let Err(_err) = expr.as_ref() {
            // TODO: This isn't always a fatal error.
            // TODO: It's weird that error messages get printed from here, right?
            // Look into this again.
            match _err {
                RloxError::Parse(ParseError::EOF) => (),
                RloxError::Parse(err) => {
                    eprintln!("Error::{err:?}");
                }
                _ => (),
            }
            self.synchronize();
        }
        expr
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        // Loop over equality expression, building up the AST with recursive Binary Expressions
        // a == b == c == d == e != f ...
        while self.is_any_tokens(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().clone(); // one of ==, !=
            let rhs = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), *operator.token_type(), Box::new(rhs));
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
            let operator = self.previous().clone(); // one of >, >=, <, <+
            let rhs = self.term()?;
            expr = Expr::Binary(Box::new(expr), *operator.token_type(), Box::new(rhs));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.is_any_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone(); // one of +, -
            let rhs = self.factor()?;
            expr = Expr::Binary(Box::new(expr), *operator.token_type(), Box::new(rhs));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.is_any_tokens(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().clone();
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), *operator.token_type(), Box::new(rhs));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.is_any_tokens(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous().clone();
            let rhs = self.unary()?;
            return Ok(Expr::Unary(*operator.token_type(), Box::new(rhs)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.is_any_tokens(&[TokenType::False]) {
            return Ok(Expr::Literal(ExprLiteral::False));
        }
        if self.is_any_tokens(&[TokenType::True]) {
            return Ok(Expr::Literal(ExprLiteral::True));
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
            self.consume(
                &TokenType::RightParen,
                "Expected ')' after expression.".into(),
            )?;

            return Ok(Expr::Grouping(Box::new(expr)));
        }

        if self.is_any_tokens(&[TokenType::Eof]) {
            return Err(RloxError::Parse(ParseError::EOF));
        }

        // Some kind of unexpected error.
        // TODO: Try to be smarter here.
        Err(RloxError::Parse(ParseError::UnexpectedToken(format!(
            "Unexpected token \"{}\" in this scope.",
            self.peek()
        ))))
    }

    // Helper functions

    fn is_any_tokens(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
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

    fn consume(&mut self, token_type: &TokenType, msg: String) -> Result<&Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(RloxError::Parse(ParseError::UnexpectedToken(msg)))
    }

    fn is_at_end(&self) -> bool {
        self.check(&TokenType::Eof)
    }

    fn check(&self, token_type: &TokenType) -> bool {
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
            if self.previous().token_type() == &TokenType::Semicolon {
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

        assert!(p.check(&TokenType::Number));

        assert_eq!(p.advance().token_type(), &TokenType::Number);
        assert_eq!(p.advance().token_type(), &TokenType::EqualEqual);
        assert_eq!(p.advance().token_type(), &TokenType::Number);
        assert_eq!(p.advance().token_type(), &TokenType::Semicolon);

        assert_eq!(p.peek().token_type(), &TokenType::Eof);
    }

    #[test]
    fn seek_past_end() {
        let mut p = get_parser_scanner(Some("var abc = 45; if (abc >= 20) { return false };"));
        while !p.is_at_end() {
            p.advance();
        }

        // we shouldn't seek past the EOF token.
        p.advance();
        assert_eq!(p.peek().token_type(), &TokenType::Eof);
        p.advance();
        assert_eq!(p.peek().token_type(), &TokenType::Eof);
    }

    #[test]
    fn basic_matching_not_equal() {
        let mut p = get_parser_scanner(None);
        assert!(!p.is_any_tokens(&[TokenType::And, TokenType::Equal, TokenType::BangEqual]));
    }

    #[test]
    fn basic_matching_equal() {
        let mut p = get_parser_scanner(None);
        assert!(p.check(&TokenType::Number));
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
