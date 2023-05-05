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

mod callable; // Traits for callable objects (functions, classes, lambdas)
mod environment; // Call stack
mod loxreturn;
mod loxtype;

use std::rc::Rc;

use crate::errors::{Result, RloxError, RuntimeError};
use crate::parser::ast::{Expr, ExprLiteral, Stmt};
use crate::scanner::TokenType;
use callable::{Callable, FfiClock, FfiPrint, LoxFunction};
use environment::RfEnv;
use loxreturn::Return;
use loxtype::LoxType;

pub struct Interpreter {
    global: RfEnv,
    env: RfEnv,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let global = environment::new_global();
        let env = Rc::clone(&global);

        environment::define(&global, "clock", LoxType::Clock(FfiClock {}));
        environment::define(&global, "print", LoxType::Print(FfiPrint {}));
        Self { global, env }
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
            Stmt::Var(ident, init) => {
                let result = match init {
                    Some(expr) => self.evaluate(expr)?,
                    None => LoxType::Nil,
                };
                environment::define(&self.env, &ident, result);
            }
            Stmt::Block(block) => {
                self.env = environment::from(&self.env);

                // Execute statements
                for stmt in block {
                    self.execute(stmt)?;
                }

                // Return to previous scope
                self.env = environment::drop(&self.env);
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
            Stmt::Fun(ident, params, body) => {
                let fun = LoxFunction::new(&ident, params, body);
                environment::define(&self.env, &ident, LoxType::Fun(fun));
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

    fn execute_block(&mut self, body: Vec<Stmt>, env: RfEnv) -> Result<Option<Return>> {
        // This function is just like execute(), but it's specific to trait Callable
        // We give it its own environment to handle 1) functions, 2) closures.
        // We also deal with return values, unlike execute().

        // Preserve the old call stack
        let old_stack = Rc::clone(&self.env);

        // Put in place the new stack
        self.env = env;

        for statement in body {
            if let Some(ret) = self.execute(statement)? {
                // If we get a Return() struct, immediately stop execution and
                // return value to caller.
                return Ok(Some(ret));
            }
        }

        // Restore the old stack
        self.env = old_stack;

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
                Ok(environment::get(&Rc::clone(&self.env), &ident)?)
            }
            // Recursively evaluate grouping's subexpressions.
            Expr::Grouping(group) => self.evaluate(*group),
            Expr::Unary(token, expr) => self.unary(token, *expr),
            Expr::Binary(expr1, token, expr2) => self.binary(*expr1, token, *expr2),
            Expr::Assign(ident, expr) => {
                // Try to evaluate the r-value
                let exprres = self.evaluate(*expr)?;
                // TODO: This clone() is really gross.
                // Assign r-value to l-value
                environment::assign(&self.env, &ident, exprres.clone())?;
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
