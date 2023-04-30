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

// TODO: Should TokenType be replaced with an AST-specific new type?
// It's used for Binary operations: +, -, /, * and Unary: -, !.
use crate::scanner::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>), // {}
    Expression(Expr), // all kinds of expressions
    Fun(
        String,     // fun name
        Vec<Token>, // fun params
        Vec<Stmt>,  // fun body
    ),
    If(
        Expr,              // condition
        Box<Stmt>,         // statement
        Option<Box<Stmt>>, // optional else statement
    ),
    Print(Expr),                 // print "a";
    Return(Token, Option<Expr>), // return a;
    Var(String, Option<Expr>),   // var declaration
    While(Expr, Box<Stmt>),      // while (true) { do_thing(); }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(String, Box<Expr>),                // a = 10;
    Binary(Box<Expr>, TokenType, Box<Expr>),  // a + a
    Call(Box<Expr>, Token, Vec<Expr>),        // doSomething();
    Grouping(Box<Expr>),                      // (a)
    Literal(ExprLiteral),                     // 3.0, "", false
    Logical(Box<Expr>, TokenType, Box<Expr>), // false or "10"
    Unary(TokenType, Box<Expr>),              // -a, !true
    Variable(String),                         // r-value
}

#[derive(Debug, Clone)]
pub enum ExprLiteral {
    Bool(bool),
    Number(f64),
    String(String),
    Nil,
}
