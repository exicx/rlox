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

use std::fmt::{Debug, Display};
use std::time::SystemTime;

use super::{environment::Environment, Interpreter, LoxType};
use crate::errors::Result;
use crate::parser::ast::Stmt;
use crate::scanner::Token;

// Callable trait defines an interface for functions, lambdas and classes
pub trait Callable: Debug + Display {
    fn arity(&self) -> u8;
    fn call(&self, interpreter: &mut Interpreter, arguments: &[LoxType]) -> Result<LoxType>;
}

// A user-defined function.
#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: String,
    closure: Environment,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}#{}()>", self.name, self.params.len())
    }
}

impl LoxFunction {
    pub fn new(name: &str, params: Vec<Token>, body: Vec<Stmt>) -> Self {
        Self {
            closure: Environment::new(),
            name: name.to_string(),
            params,
            body,
        }
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> u8 {
        self.params.len() as u8
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: &[LoxType]) -> Result<LoxType> {
        assert_eq!(self.params.len(), arguments.len());

        // Create an environment, branched from the global environment
        // In that environment, bind the arguments to the parameters from
        // the function declaration.

        // This is the worst thing I've ever done
        // TODO: This doesn't even work.
        let mut env = interpreter.env.clone();
        while env.drop() {}
        env.new_scope();

        // Zip up arguments and their results
        // Bind each value to its name in the new environment

        let items = self.params.iter().zip(arguments.iter());

        for (token, loxtype) in items {
            env.define(token.lexeme(), loxtype.clone());
        }

        // Execute function and return its (optional) return value
        match interpreter.execute_block(self.body.clone(), env)? {
            Some(ret) => Ok(ret.0),
            None => Ok(LoxType::Nil),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FfiClock;

impl Display for FfiClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Callable for FfiClock {
    fn arity(&self) -> u8 {
        0
    }
    fn call(&self, _: &mut Interpreter, _: &[LoxType]) -> Result<LoxType> {
        Ok(LoxType::Number(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as f64,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct FfiPrint;
impl Display for FfiPrint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")
    }
}

impl Callable for FfiPrint {
    fn arity(&self) -> u8 {
        1
    }
    fn call(&self, _: &mut Interpreter, arguments: &[LoxType]) -> Result<LoxType> {
        println!("{}", arguments[0]);
        Ok(LoxType::Nil)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_basics_callable() {
        todo!();
    }
}
