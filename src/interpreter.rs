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

use std::fmt;

use crate::errors::RloxError;
use crate::parser::ast::{Expr, ExprLiteral};
use crate::tokens::TokenType;

#[derive(Debug, PartialEq)]
pub enum ExprResult {
    Nil,
    Bool(bool),
    Number(f64),
    LoxString(String),
}

impl ExprResult {
    pub fn interpret(expr: Expr) -> Result<ExprResult, RloxError> {
        evaluate(expr)
    }
}

impl fmt::Display for ExprResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out: String = match self {
            ExprResult::Bool(v) => {
                if *v {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            ExprResult::Nil => "nil".into(),
            ExprResult::Number(n) => n.to_string(),
            ExprResult::LoxString(s) => s.clone(),
        };
        write!(f, "{}", out)
    }
}

fn evaluate(expr: Expr) -> Result<ExprResult, RloxError> {
    match expr {
        // Evaluate literals
        Expr::Literal(lit) => match lit {
            ExprLiteral::False => Ok(ExprResult::Bool(false)),
            ExprLiteral::True => Ok(ExprResult::Bool(true)),
            ExprLiteral::Nil => Ok(ExprResult::Nil),
            ExprLiteral::Number(n) => Ok(ExprResult::Number(n)),
            ExprLiteral::String(ls) => Ok(ExprResult::LoxString(ls)),
        },
        // Recursively evaluate grouping's subexpressions.
        Expr::Grouping(group) => evaluate(*group),
        Expr::Unary(token, expr) => unary(token, *expr),
        Expr::Binary(expr1, token, expr2) => binary(*expr1, token, *expr2),
    }
}

// TODO: This is pretty sloppy. Cleanup this logic.
// We have two tokens, ! and -, and two possible types Number and Bool.
// Evaluate the 4 possible inputs.
fn unary(token: TokenType, unary: Expr) -> Result<ExprResult, RloxError> {
    if token != TokenType::Bang && token != TokenType::Minus {
        unimplemented!(
            "Interpreter does not support this unary operator: {:?}",
            token
        );
    }

    let right: ExprResult = evaluate(unary)?;
    match right {
        ExprResult::LoxString(_) | ExprResult::Nil => {
            return Err(RloxError::Interpret(format!(
                "Cannot apply unary operator \"{token:?}\" to expression."
            )));
        }
        _ => (),
    }

    match token {
        TokenType::Bang => Ok(ExprResult::Bool(!is_truthy(&right))),
        TokenType::Minus => {
            if let ExprResult::Number(n) = right {
                Ok(ExprResult::Number(-n))
            } else if let ExprResult::Bool(v) = right {
                Ok(ExprResult::Bool(!v))
            } else {
                unimplemented!("Not possible to get here.")
            }
        }
        _ => {
            unimplemented!("Not possible to get here.")
        }
    }
}

fn binary(expr1: Expr, token: TokenType, expr2: Expr) -> Result<ExprResult, RloxError> {
    use self::ExprResult::{Bool, LoxString, Number};

    let left = evaluate(expr1)?;
    let right = evaluate(expr2)?;

    match token {
        TokenType::Minus => {
            if let (Number(left), Number(right)) = (left, right) {
                Ok(Number(left - right))
            } else {
                Err(RloxError::Interpret("Cannot subtract types".into()))
            }
        }
        TokenType::Slash => {
            if let (Number(left), Number(right)) = (left, right) {
                Ok(Number(left / right))
            } else {
                Err(RloxError::Interpret("Cannot divide types".into()))
            }
        }
        TokenType::Star => {
            if let (Number(left), Number(right)) = (left, right) {
                Ok(Number(left * right))
            } else {
                Err(RloxError::Interpret("Cannot multiply types".into()))
            }
        }
        TokenType::Plus => {
            // Handle number addition
            if let (Number(left), Number(right)) = (&left, &right) {
                Ok(Number(left + right))
            }
            // Handle string concatenation
            else if let (LoxString(left), LoxString(right)) = (left, right) {
                Ok(LoxString(left + &right))
            } else {
                Err(RloxError::Interpret("Cannot concatenate types".into()))
            }
        }

        TokenType::Greater => {
            if let (Number(left), Number(right)) = (left, right) {
                Ok(Bool(left > right))
            } else {
                Err(RloxError::Interpret("Cannot compare types".into()))
            }
        }
        TokenType::GreaterEqual => {
            if let (Number(left), Number(right)) = (left, right) {
                Ok(Bool(left >= right))
            } else {
                Err(RloxError::Interpret("Cannot compare types".into()))
            }
        }
        TokenType::Less => {
            if let (Number(left), Number(right)) = (left, right) {
                Ok(Bool(left < right))
            } else {
                Err(RloxError::Interpret("Cannot compare types".into()))
            }
        }
        TokenType::LessEqual => {
            if let (Number(left), Number(right)) = (left, right) {
                Ok(Bool(left <= right))
            } else {
                Err(RloxError::Interpret("Cannot compare types".into()))
            }
        }

        TokenType::EqualEqual => Ok(Bool(is_equal(&left, &right))),
        TokenType::BangEqual => Ok(Bool(!is_equal(&left, &right))),

        _ => {
            unimplemented!("No other binary tokens are implemented.")
        }
    }
}

fn is_truthy(expr: &ExprResult) -> bool {
    match expr {
        ExprResult::Bool(v) => *v,
        ExprResult::Nil => false,
        _ => true,
    }
}

fn is_equal(left: &ExprResult, right: &ExprResult) -> bool {
    *left == *right
}

// errors
