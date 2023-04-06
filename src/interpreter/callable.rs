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

use super::{Interpreter, LoxType};
use crate::errors::{Result, RloxError, RuntimeError};

// Callable trait defines an interface for functions, lambdas and classes
pub(super) trait Callable {
    fn arity(&self) -> u16;
    fn call(&self, interpreter: &mut Interpreter, arguments: &[LoxType]) -> Result<LoxType>;
}

pub(super) struct Function {
    name: String,
    arity: u16,
}

impl Function {
    fn new(name: String, arity: u16) -> Self {
        Self { name, arity }
    }
}

impl Callable for Function {
    fn arity(&self) -> u16 {
        self.arity
    }
    fn call(&self, interpreter: &mut Interpreter, arguments: &[LoxType]) -> Result<LoxType> {
        eprintln!("Executed function: {}", self.name);

        Ok(LoxType::Nil)
    }
}

impl TryFrom<LoxType> for Function {
    type Error = RloxError;
    fn try_from(value: LoxType) -> std::result::Result<Self, Self::Error> {
        match value {
            // TODO: change this from String to Function type once implemented.
            LoxType::String(fun) => Ok(Function::new(fun, 0)),
            // LoxType::Function() => {},
            // LoxType::Class() => {},
            _ => Err(RloxError::Interpret(RuntimeError::NotACallableType(
                "Can only call functions and classes.".to_string(),
            ))),
        }
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
