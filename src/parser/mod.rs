// rlox: Lox interpreter/compiler in Rust.
// Copyright (C) 2023  James Smyle <j@mes.sh>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod ast;

use crate::errors::{ParseError, Result, RloxError};
use crate::scanner::{Token, TokenLiteral, TokenType};
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
        } else if self.is_any_tokens(&[TokenType::Fun]) {
            self.function("function")
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

    // Handle function declaration
    fn function(&mut self, kind: &str) -> Result<Stmt> {
        // function identifier
        let name = self
            .consume(TokenType::Identifier, &format!("Expect {} name.", kind))?
            .clone();

        // parameter
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut params = vec![];

        loop {
            if self.check(TokenType::RightParen) {
                // break when we find closing )
                break;
            }

            if params.len() >= 255 {
                // error if there's too many params
                return Err(RloxError::Parse(ParseError::TooManyParameters));
            }

            // add param identifier to list
            params.push(
                self.consume(TokenType::Identifier, "Expect parameter name.")?
                    .clone(),
            );

            if !self.is_any_tokens(&[TokenType::Comma]) {
                // break if there's no more params
                break;
            }
        }

        // end of parameters
        self.consume(
            TokenType::RightParen,
            &format!("Expect ')' after {} paramaters.", kind),
        )?;

        // Parse out function body in a block
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;

        let body = self.block_stmt()?;

        Ok(Stmt::Fun(name.token_literal().to_string(), params, body))
    }

    // Statement functions

    fn statement(&mut self) -> Result<Stmt> {
        if self.is_any_tokens(&[TokenType::If]) {
            // If condition
            self.if_stmt()
        } else if self.is_any_tokens(&[TokenType::LeftBrace]) {
            // New block/scope
            Ok(Stmt::Block(self.block_stmt()?))
        } else if self.is_any_tokens(&[TokenType::While]) {
            // While loop
            self.while_stmt()
        } else if self.is_any_tokens(&[TokenType::For]) {
            // For loop
            self.for_stmt()
        } else if self.is_any_tokens(&[TokenType::Return]) {
            self.return_stmt()
        } else {
            // We're an expression
            self.expression_stmt()
        }
    }

    // block_stmt returns a vector of statements NOT enclosed within a
    // stmt::block.
    fn block_stmt(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(stmts)
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

        // TODO we create an extra Block statement (and create a new environment)
        // because we desguar for-loops into a statement like
        // block {
        //   block {
        //     initializer;
        //     while(condition) block {
        //        body;
        //        increment;
        //     }
        //   }
        // }
        //
        // this is unnecessary but it's not "wrong." it does slow down variable resolution
        Ok(body)
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        let keyword = self.previous().clone();
        let expr = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return(keyword, expr))
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

    // a = "10";
    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.is_any_tokens(&[TokenType::Equal]) {
            let value = self.assignment()?;

            return if let Expr::Variable(name, _) = expr {
                Ok(Expr::Assign(name, Box::new(value), None))
            } else {
                Err(RloxError::Parse(ParseError::ParseFailure(
                    "Invalid assignment target.".to_string(),
                )))
            };
        }

        Ok(expr)
    }

    // a or b
    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.is_any_tokens(&[TokenType::Or]) {
            let operator = self.previous().token_type();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // a and b
    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.is_any_tokens(&[TokenType::And]) {
            let operator = self.previous().token_type();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    // a == b, true != false
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

    // a > b
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

    // a + b, a - b
    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.is_any_tokens(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().token_type(); // one of +, -
            let rhs = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    // a * b, a / b
    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.is_any_tokens(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous().token_type();
            let rhs = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(rhs));
        }

        Ok(expr)
    }

    // !b, -a
    fn unary(&mut self) -> Result<Expr> {
        if self.is_any_tokens(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous().token_type();
            let rhs = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(rhs)));
        }

        self.call()
    }

    // do_something()
    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.is_any_tokens(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    // false, true, nil, groupings, identifiers, strings, numbers
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
                return Ok(Expr::Variable(literal, None));
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

    // returns true if the next token is any of the ones we're searching for
    fn is_any_tokens(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(*token) {
                self.advance();
                return true;
            }
        }
        false
    }

    // advances to the next token and returns a ref to it
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // checks the next token is the kind we're looking for, and advances if so.
    // otherwise, return the given error message in an UnexpectedToken type
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

    // skips past a syntacically invalid statement and finds the beginning
    // of the next one based on a few tokens: class, fun, var, for, ...
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
                | TokenType::Return => return,
                _ => {}
            }
            self.advance();
        }
    }

    // the second half of the call() function
    // parses the arguments and consumes the ending ')'.
    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = vec![];

        // Add zero or more arguments to vec
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(RloxError::Parse(ParseError::TooManyParameters));
                }
                arguments.push(self.expression()?);

                if !self.is_any_tokens(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        // Consume ending ')'
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(Expr::Call(Box::new(callee), paren.clone(), arguments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::TokenType;

    fn get_scanner(opt: Option<&str>) -> crate::scanner::Scanner {
        let mut scanner = crate::scanner::Scanner::new();

        match opt {
            Some(x) => scanner.scan_tokens(x).unwrap(),
            None => scanner.scan_tokens("10 == 20;").unwrap(),
        };

        scanner
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
