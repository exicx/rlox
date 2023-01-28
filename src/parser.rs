mod ast {
    use crate::tokens::Token;

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
        Binary(Box<Expr>, Token, Box<Expr>),
        Unary(Token, Box<Expr>),
        Literal(ExprLiteral),
    }
}

use self::ast::{Expr, ExprLiteral};
use crate::errors::{self, RloxError};
use crate::tokens::{Token, TokenLiteral, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        Parser {
            tokens: tokens.to_vec(),
            current: 0,
        }
    }

    // Build up the AST by precendence

    pub fn parse(&mut self) -> Option<Expr> {
        if let Ok(expr) = self.expression() {
            Some(expr)
        } else {
            None
        }
    }

    fn expression(&mut self) -> Result<Expr, errors::RloxError> {
        let expr = self.equality();
        if let Err(err) = expr.as_ref() {
            eprintln!("Error:: {err:?}");
            self.synchronize();
        }
        expr
    }

    fn equality(&mut self) -> Result<Expr, errors::RloxError> {
        let mut expr = self.comparison()?;

        // Loop over equality expression, building up the AST with recursive Binary Expressions
        // a == b == c == d == e != f ...
        while self.is_any_tokens(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().clone(); // one of ==, !=
            let rhs = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, errors::RloxError> {
        let mut expr = self.term()?;

        while self.is_any_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone(); // one of >, >=, <, <+
            let rhs = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, errors::RloxError> {
        let mut expr = self.factor()?;

        while self.is_any_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().clone(); // one of +, -
            let rhs = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, errors::RloxError> {
        let mut expr = self.unary()?;

        while self.is_any_tokens(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().clone();
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, errors::RloxError> {
        if self.is_any_tokens(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous().clone();
            let rhs = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(rhs)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, errors::RloxError> {
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

        Err(errors::RloxError::Parse(
            "Unexpected token in this scope.".into(),
        ))
    }

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

    fn consume(&mut self, token_type: &TokenType, msg: String) -> Result<&Token, RloxError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(errors::RloxError::Parse(msg))
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
        Parser::new(get_scanner(opt).get_tokens())
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
