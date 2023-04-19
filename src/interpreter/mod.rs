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

mod callable; // Traits for callable objects (functions, classes, lambdas)
mod environment; // Call stack
mod loxreturn;
mod loxtype;

use crate::errors::{Result, RloxError, RuntimeError};
use crate::parser::ast::{Expr, ExprLiteral, Stmt};
use crate::scanner::TokenType;
use callable::{Callable, FfiClock, FfiPrint, LoxFunction};
use environment::Environment;
use loxreturn::Return;
use loxtype::LoxType;

pub struct Interpreter {
    env: Environment,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut env = Environment::new();
        env.define("clock", LoxType::Clock(FfiClock {}));
        env.define("print", LoxType::Print(FfiPrint {}));
        Self { env }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Default::default()
    }

    //
    // Handling programs and statements
    //

    pub fn interpret(&mut self, program: Vec<Stmt>) -> Result<()> {
        for statement in program {
            self.execute(statement)?;
            // we don't need the Return() type here, only inside
            // functions/closures/methods
        }

        Ok(())
    }

    fn execute(&mut self, stmt: Stmt) -> Result<Option<Return>> {
        match stmt {
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
                    None => LoxType::Nil,
                };
                self.env.define(&identifier, result);
            }
            Stmt::Block(block) => {
                self.env.new_scope();

                // Execute statements
                for stmt in block {
                    self.execute(stmt)?;
                }

                // Return to previous scope
                self.env.drop();
            }
            Stmt::If(condition, then_branch, else_branch) => {
                if is_truthy(&self.evaluate(condition)?) {
                    self.execute(*then_branch)?;
                } else if else_branch.is_some() {
                    self.execute(*else_branch.unwrap())?;
                }
            }
            // TODO: Look into this.
            // Can we do something besides clone() to re-evaluate the condition?
            Stmt::While(condition, stmt) => {
                while is_truthy(&self.evaluate(condition.clone())?) {
                    self.execute(*stmt.clone())?;
                }
            }
            Stmt::Fun(name, params, body) => {
                let fun = LoxFunction::new(&name, params, body);
                self.env.define(&name, LoxType::Fun(fun));
            }
            // TODO: Use token to improve interpreter error messages.
            Stmt::Return(_tok, expr) => {
                let val = match expr {
                    Some(expr) => self.evaluate(expr)?,
                    None => LoxType::Nil,
                };

                return Ok(Some(Return(val)));
            }
        }

        Ok(None)
    }

    fn execute_block(
        &mut self,
        body: Vec<Stmt>,
        environment: Environment,
    ) -> Result<Option<Return>> {
        // This function is just like execute(), but it's specific to trait Callable
        // We give it its own environment to handle 1) functions, 2) closures.
        // We also deal with return values that execute() ignores.

        // TODO: Handle environment.

        for statement in body {
            if let Some(ret) = self.execute(statement)? {
                // If we get a Return() struct, immediately stop execution and
                // return value to caller.
                return Ok(Some(ret));
            }
        }

        Ok(None)
    }

    //
    // Handling Expressions
    //

    fn evaluate(&mut self, expr: Expr) -> Result<LoxType> {
        match expr {
            // Evaluate literals
            Expr::Literal(lit) => match lit {
                ExprLiteral::Bool(v) => Ok(LoxType::Bool(v)),
                ExprLiteral::Nil => Ok(LoxType::Nil),
                ExprLiteral::Number(n) => Ok(LoxType::Number(n)),
                ExprLiteral::String(ls) => Ok(LoxType::String(ls)),
            },
            Expr::Variable(ident) => {
                // Accessing a variable.
                Ok(self.env.get(&ident)?.clone())
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
                self.env.assign(&name, exprres.clone())?;
                Ok(exprres)
            }
            Expr::Logical(left, operator, right) => {
                let left = self.evaluate(*left)?;

                // short-circuit. only evaluate the right if needed.

                if operator == TokenType::Or {
                    if is_truthy(&left) {
                        // operator == or, and left is true
                        // so return true
                        return Ok(left);
                    }
                } else if !is_truthy(&left) {
                    // operator == and, but left is false
                    // so return false
                    return Ok(left);
                }

                // otherwise, return whatever the right side is after evaluating it.
                Ok(self.evaluate(*right)?)
            }
            // TODO: Use token to improve interpreter error messages.
            Expr::Call(callee, _tok, arguments) => {
                let callee = self.evaluate(*callee)?;
                let call: Box<dyn Callable> = match callee {
                    LoxType::Fun(callee) => Box::new(callee),
                    LoxType::Clock(callee) => Box::new(callee),
                    other => {
                        return Err(RloxError::Interpret(RuntimeError::NotACallableType(
                            other.to_string(),
                        )));
                    }
                };

                // Evaluate arguments of function call
                let mut args = vec![];
                for argument in arguments {
                    args.push(self.evaluate(argument)?);
                }

                // Make sure we have the correct number of arguments,
                // Then call the function
                if call.arity() as usize != args.len() {
                    Err(RloxError::Interpret(RuntimeError::MismatchedArguments(
                        format!(
                            "Expected {} arguments, but got {}.",
                            call.arity(),
                            args.len()
                        ),
                    )))
                } else {
                    Ok(call.call(self, &args)?)
                }
            }
        }
    }

    // TODO: This is pretty sloppy. Cleanup this logic.
    // We have two tokens, ! and -, and two possible types Number and Bool.
    // Evaluate the 4 possible inputs.
    fn unary(&mut self, token: TokenType, unary: Expr) -> Result<LoxType> {
        if token != TokenType::Bang && token != TokenType::Minus {
            unimplemented!(
                "Interpreter does not support this unary operator: {:?}",
                token
            );
        }

        let right: LoxType = self.evaluate(unary)?;
        match right {
            LoxType::String(_) | LoxType::Nil => {
                return Err(RloxError::Interpret(RuntimeError::TypeComparison(format!(
                    "Cannot apply unary operator \"{token:?}\" to expression."
                ))));
            }
            _ => (),
        }

        match token {
            TokenType::Bang => Ok(LoxType::Bool(!is_truthy(&right))),
            TokenType::Minus => {
                if let LoxType::Number(n) = right {
                    Ok(LoxType::Number(-n))
                } else if let LoxType::Bool(v) = right {
                    Ok(LoxType::Bool(!v))
                } else {
                    unimplemented!("Not possible to get here.")
                }
            }
            _ => {
                unimplemented!("Not possible to get here.")
            }
        }
    }

    fn binary(&mut self, expr1: Expr, token: TokenType, expr2: Expr) -> Result<LoxType> {
        use self::LoxType::{Bool, Number, String};

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

fn is_truthy(expr: &LoxType) -> bool {
    match expr {
        LoxType::Bool(v) => *v,
        LoxType::Nil => false,
        _ => true,
    }
}

fn is_equal(left: &LoxType, right: &LoxType) -> bool {
    if let (LoxType::Bool(v1), LoxType::Bool(v2)) = (left, right) {
        v1 == v2
    } else if let (LoxType::Nil, LoxType::Nil) = (left, right) {
        true
    } else if let (LoxType::Number(n1), LoxType::Number(n2)) = (left, right) {
        n1 == n2
    } else if let (LoxType::String(s1), LoxType::String(s2)) = (left, right) {
        s1 == s2
    } else {
        false
    }

    // TODO figure out how to compare Fun and Class types
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_interpreter() {
        todo!()
    }
}
