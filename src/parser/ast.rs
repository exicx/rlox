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

// TODO: Should TokenType be replaced with an AST-specific new type?
// It's used for Binary operations: +, -, /, * and Unary: -, !.
use crate::tokens::TokenType;

#[derive(Debug)]
pub enum ExprLiteral {
    Bool(bool),
    Number(f64),
    String(String),
    Nil,
}

#[derive(Debug)]
pub enum Expr {
    Assign(String, Box<Expr>),                // a = 10;
    Binary(Box<Expr>, TokenType, Box<Expr>),  // a + a
    Grouping(Box<Expr>),                      // (a)
    Literal(ExprLiteral),                     // 3.0, "", false
    Logical(Box<Expr>, TokenType, Box<Expr>), // false or "10"
    Unary(TokenType, Box<Expr>),              // -a, !true
    Variable(String),                         // r-value
}

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>), // {}
    Expression(Expr), // all kinds of expressions
    If(
        Expr,              // condition
        Box<Stmt>,         // statement
        Option<Box<Stmt>>, // optional else statement
    ), // if (true) print a;
    Print(Expr),      // print "a";
    Var(String, Option<Expr>), // var declaration
}
