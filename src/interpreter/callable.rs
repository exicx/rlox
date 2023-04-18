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

use std::fmt::{Debug, Display};
use std::time::SystemTime;

use super::{Interpreter, LoxType};
use crate::errors::Result;
use crate::parser::ast::Stmt;
use crate::tokens::Token;

// Callable trait defines an interface for functions, lambdas and classes
pub(super) trait Callable: Debug + Display {
    fn arity(&self) -> u8;
    fn call(&self, interpreter: &mut Interpreter, arguments: &[LoxType]) -> Result<LoxType>;
}

// A user-defined function.
#[derive(Debug, Clone)]
pub(super) struct LoxFunction {
    name: String,
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
pub(super) struct FfiClock;

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
pub(super) struct FfiPrint;
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
