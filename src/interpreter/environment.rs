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

use std::collections::HashMap;

use crate::errors::{RloxError, RuntimeError};

use super::ExprResult;

#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<String, ExprResult>,
}

impl Environment {
    pub fn define(&mut self, name: String, res: ExprResult) {
        self.values.insert(name, res);
    }

    pub fn undefine(&mut self, name: &str) {
        self.values.remove(name);
    }

    pub fn get(&self, name: &str) -> Result<&ExprResult, RloxError> {
        match self.values.get(name) {
            None => Err(RloxError::Interpret(RuntimeError::UndefinedVariable)),
            Some(res) => Ok(res),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let mut env = Environment::default();

        env.define("var1".to_string(), ExprResult::Bool(true));
        env.define("var2".to_string(), ExprResult::Nil);

        assert_eq!(env.get("var2"), Ok(&ExprResult::Nil));
        assert_eq!(env.get("var1"), Ok(&ExprResult::Bool(true)));
    }
}
