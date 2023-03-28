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
    Grouping(Box<Expr>),
    Binary(Box<Expr>, TokenType, Box<Expr>),
    Unary(TokenType, Box<Expr>),
    Literal(ExprLiteral),
    Variable(String),
}

#[derive(Debug)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    Var(String, Option<Expr>),
}
