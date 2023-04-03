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

mod environment;

use std::fmt;

use crate::errors::{RloxError, RuntimeError};
use crate::parser::ast::{Expr, ExprLiteral, Stmt};
use crate::tokens::TokenType;
use environment::Environment;

// TODO why am I doing this?
#[derive(Debug, PartialEq, Clone)]
pub enum ExprResult {
    Bool(bool),
    Number(f64),
    String(String),
    Nil,
}

impl fmt::Display for ExprResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprResult::Bool(v) => {
                if *v {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            ExprResult::Nil => write!(f, "nil"),
            ExprResult::Number(n) => write!(f, "{}", n),
            ExprResult::String(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Default)]
pub struct Interpreter {
    // TODO some kind of stack or state tracking
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Default::default()
    }
    pub fn interpret(&mut self, program: Vec<Stmt>) -> Result<(), RloxError> {
        for statement in program {
            match statement {
                Stmt::Expression(expr) => {
                    self.evaluate(expr)?;
                }
                Stmt::Print(expr) => {
                    let result = self.evaluate(expr)?;
                    println!("{result}");
                }
                Stmt::Var(identifier, initializer) => {
                    let result = match initializer {
                        Some(expr) => self.evaluate(expr)?,
                        None => ExprResult::Nil,
                    };
                    self.environment.define(&identifier, result);
                }
            }
        }

        Ok(())
    }

    fn evaluate(&mut self, expr: Expr) -> Result<ExprResult, RloxError> {
        match expr {
            // Evaluate literals
            // ? TODO
            // Why am I converting from an ExprLiteral to an ExprResult
            Expr::Literal(lit) => match lit {
                ExprLiteral::Bool(v) => Ok(ExprResult::Bool(v)),
                ExprLiteral::Nil => Ok(ExprResult::Nil),
                ExprLiteral::Number(n) => Ok(ExprResult::Number(n)),
                ExprLiteral::String(ls) => Ok(ExprResult::String(ls)),
            },
            Expr::Variable(ident) => {
                // Accessing a variable.
                Ok(self.environment.get(&ident)?.clone())
            }
            // Recursively evaluate grouping's subexpressions.
            Expr::Grouping(group) => self.evaluate(*group),
            Expr::Unary(token, expr) => self.unary(token, *expr),
            Expr::Binary(expr1, token, expr2) => self.binary(*expr1, token, *expr2),
            Expr::Assign(name, expr) => {
                // Try to evaluate the r-value
                let exprres = self.evaluate(*expr)?;
                // TODO: This clone() is really gross.
                // Assign r-value to l-value
                self.environment.assign(&name, exprres.clone())?;
                Ok(exprres)
            }
        }
    }

    // TODO: This is pretty sloppy. Cleanup this logic.
    // We have two tokens, ! and -, and two possible types Number and Bool.
    // Evaluate the 4 possible inputs.
    fn unary(&mut self, token: TokenType, unary: Expr) -> Result<ExprResult, RloxError> {
        if token != TokenType::Bang && token != TokenType::Minus {
            unimplemented!(
                "Interpreter does not support this unary operator: {:?}",
                token
            );
        }

        let right: ExprResult = self.evaluate(unary)?;
        match right {
            ExprResult::String(_) | ExprResult::Nil => {
                return Err(RloxError::Interpret(RuntimeError::TypeComparison(format!(
                    "Cannot apply unary operator \"{token:?}\" to expression."
                ))));
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

    fn binary(
        &mut self,
        expr1: Expr,
        token: TokenType,
        expr2: Expr,
    ) -> Result<ExprResult, RloxError> {
        use self::ExprResult::{Bool, Number, String};

        let left = self.evaluate(expr1)?;
        let right = self.evaluate(expr2)?;

        match token {
            TokenType::Minus => {
                if let (Number(left), Number(right)) = (left, right) {
                    Ok(Number(left - right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::Arithmetic(
                        "Cannot subtract types".into(),
                    )))
                }
            }
            TokenType::Slash => {
                if let (Number(left), Number(right)) = (left, right) {
                    Ok(Number(left / right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::Arithmetic(
                        "Cannot divide types".into(),
                    )))
                }
            }
            TokenType::Star => {
                if let (Number(left), Number(right)) = (left, right) {
                    Ok(Number(left * right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::Arithmetic(
                        "Cannot multiply types".into(),
                    )))
                }
            }
            TokenType::Plus => {
                // Handle number addition
                if let (Number(left), Number(right)) = (&left, &right) {
                    Ok(Number(left + right))
                }
                // Handle string concatenation
                else if let (String(left), String(right)) = (left, right) {
                    Ok(String(left + &right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::Concatenation(
                        "Cannot concatenate types".into(),
                    )))
                }
            }

            TokenType::Greater => {
                if let (Number(left), Number(right)) = (left, right) {
                    Ok(Bool(left > right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::TypeComparison(
                        "Cannot compare types".into(),
                    )))
                }
            }
            TokenType::GreaterEqual => {
                if let (Number(left), Number(right)) = (left, right) {
                    Ok(Bool(left >= right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::TypeComparison(
                        "Cannot compare types".into(),
                    )))
                }
            }
            TokenType::Less => {
                if let (Number(left), Number(right)) = (left, right) {
                    Ok(Bool(left < right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::TypeComparison(
                        "Cannot compare types".into(),
                    )))
                }
            }
            TokenType::LessEqual => {
                if let (Number(left), Number(right)) = (left, right) {
                    Ok(Bool(left <= right))
                } else {
                    Err(RloxError::Interpret(RuntimeError::TypeComparison(
                        "Cannot compare types".into(),
                    )))
                }
            }

            TokenType::EqualEqual => Ok(Bool(is_equal(&left, &right))),
            TokenType::BangEqual => Ok(Bool(!is_equal(&left, &right))),

            _ => {
                unimplemented!("No other binary tokens are implemented.")
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter() {
        todo!()
    }
}
