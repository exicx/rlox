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

// Depth of this variable's usage in the call stack.
// Used for quick lookups in interpreter::environment,
// and set via the semantic analysis pass: resolver.
type EnvDepth = u32;

// TODO: Replace String and TokenType types with Token.
// Give `Token`s to the interpreter to print better error messages.

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
    Return(Token, Option<Expr>), // return a;
    Var(String, Option<Expr>),   // "var" x (= 10)? ;
    While(Expr, Box<Stmt>),      // while (true) { do_thing(); }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(String, Box<Expr>, Option<EnvDepth>), // a = 10;
    Binary(Box<Expr>, TokenType, Box<Expr>),     // a + a
    Call(Box<Expr>, Token, Vec<Expr>),           // doSomething();
    Grouping(Box<Expr>),                         // (a)
    Literal(ExprLiteral),                        // 3.0, "", false
    Logical(Box<Expr>, TokenType, Box<Expr>),    // false or "10"
    Unary(TokenType, Box<Expr>),                 // -a, !true
    Variable(String, Option<EnvDepth>),          // r-value
}

#[derive(Debug, Clone)]
pub enum ExprLiteral {
    Bool(bool),
    Number(f64),
    String(String),
    Nil,
}

//
// WIP
//
//

// impl std::fmt::Display for Stmt {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Stmt::Block(ref v) => {
//                 writeln!(f, "block: ");
//                 for stmt in v {
//                     writeln!(f, "  {}", stmt);
//                 }
//             }
//             Stmt::Expression(expr) => {
//                 writeln!(f, "{}", expr);
//             }
//         }

//         writeln!(f, "");
//         Ok(())
//     }
// }

// impl std::fmt::Display for Expr {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Expr::Assign(s, expr) => {
//                 write!(f, "{} = {}", s, expr)
//             }
//             Expr::Binary(a, tt, b) => {
//                 write!(f, "{} {:?} {}", a, tt, b)
//             }
//             Expr::Call(name, _, params) => {
//                 write!(f, "{}({})", name, params)
//             }
//         }
//     }
// }

// impl std::fmt::Display for ExprLiteral {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ExprLiteral::Bool(b) => {
//                 if *b {
//                     write!(f, "true")
//                 } else {
//                     write!(f, "false")
//                 }
//             }
//             ExprLiteral::Number(n) => write!(f, "{}", n),
//             ExprLiteral::String(s) => write!(f, "{}", s),
//             ExprLiteral::Nil => write!(f, "nil"),
//         }
//     }
// }
