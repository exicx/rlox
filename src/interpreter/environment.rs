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
pub(super) struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, ExprResult>,
}

impl Environment {
    // Create a new scope that links to the previous one.
    pub fn new_scope(self) -> Environment {
        Environment {
            enclosing: Some(Box::new(self)),
            ..Default::default()
        }
    }

    // Drop the top-most scope
    pub fn drop(mut self) -> Environment {
        match self.enclosing.take() {
            Some(e) => *e,
            None => Environment::default(),
        }
    }

    pub fn define(&mut self, name: &str, res: ExprResult) {
        self.values.insert(name.to_string(), res);
    }

    // Drop value
    #[allow(dead_code)]
    pub fn undefine(&mut self, name: &str) {
        self.values.remove(name);
    }

    // Return value if it exists, otherwise error
    // Recurses up call stack
    pub fn get(&self, name: &str) -> Result<&ExprResult, RloxError> {
        match self.values.get(name) {
            Some(res) => Ok(res),
            None => {
                // Check if any nested scopes have the variable name.
                match &self.enclosing {
                    Some(enclosing) => enclosing.get(name),
                    None => Err(RloxError::Interpret(RuntimeError::UndefinedVariable)),
                }
            }
        }
    }

    pub fn assign(&mut self, name: &str, res: ExprResult) -> Result<(), RloxError> {
        // Check if value already exists, if it doesn't then return error
        self.get(name)?;
        // Otherwise, update value and return success
        self.define(name, res);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let mut env = Environment::default();

        env.define("var1", ExprResult::Bool(true));
        env.define("var2", ExprResult::Nil);

        assert_eq!(env.get("var2"), Ok(&ExprResult::Nil));
        assert_eq!(env.get("var1"), Ok(&ExprResult::Bool(true)));
    }

    #[test]
    fn test_assignment() {
        let mut env = Environment::default();

        env.define("var_test", ExprResult::Bool(true));
        match env.assign("var_test", ExprResult::Bool(false)) {
            Ok(_) => (),
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_failing_assignment() {
        let mut env = Environment::default();

        env.define("fail_me", ExprResult::Bool(true));

        // Fails because "var_test" was not previously defined
        match env.assign("var_test", ExprResult::Bool(false)) {
            Ok(_) => (),
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    #[test]
    fn test_nested_get() {
        let mut env1 = Environment::default();
        env1.define("name1", ExprResult::Bool(true));
        env1.define("name2", ExprResult::Bool(false));

        let mut env2 = env1.new_scope();
        env2.define("name3", ExprResult::String("Found".to_string()));

        let env3 = env2.new_scope();

        if env3.get("name1").is_err() {
            panic!("Nested environments did not work");
        }
        if env3.get("name2").is_err() {
            panic!("Nested environments did not work");
        }

        if env3.get("name3").is_err() {
            panic!("Nested environments did not work");
        }
    }

    #[test]
    fn test_nested_assignment() {
        let mut env1 = Environment::default();
        env1.define("name1", ExprResult::Bool(true));
        env1.define("name2", ExprResult::Bool(false));

        let mut env2 = env1.new_scope();
        env2.define("name3", ExprResult::String("Found".to_string()));

        let mut env3 = env2.new_scope();

        if env3.assign("name4", ExprResult::Number(32.)).is_ok() {
            panic!("Shouldn't assign to unknown variable.");
        }

        if env3.assign("name1", ExprResult::Nil).is_err() {
            panic!("Should assign to known variable in nested scope.");
        }

        let env2 = env3.drop();
        let env1 = env2.drop();

        if env1.get("name1").unwrap() != &ExprResult::Nil {
            panic!("Did not overwrite variable in parent scope.");
        }
    }
}
