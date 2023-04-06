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

use crate::errors::{Result, RloxError, RuntimeError};

use super::LoxType;

#[derive(Debug, Default)]
struct Scope {
    values: HashMap<String, LoxType>,
}

#[derive(Debug)]
pub(super) struct Environment {
    scopes: Vec<Scope>,
}

impl Default for Environment {
    fn default() -> Self {
        let mut scopes = Vec::new();
        scopes.reserve(8);
        scopes.push(Scope::default());

        Self { scopes }
    }
}

impl Environment {
    // Create a new scope at the end of the queue
    pub fn new_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    // Drop the top-most scope
    // self.scopes[0] is the global scope, don't allow dropping
    // that one.
    pub fn drop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    // Define a new type
    pub fn define(&mut self, name: &str, res: LoxType) {
        self.scopes
            .last_mut()
            .unwrap()
            .values
            .insert(name.to_string(), res);
    }

    // Drop value
    #[allow(dead_code)]
    pub fn undefine(&mut self, name: &str) {
        self.scopes.last_mut().unwrap().values.remove(name);
    }

    // Return value if it exists, otherwise error
    // Recurses up call stack
    pub fn get(&self, name: &str) -> Result<&LoxType> {
        let stack_iter = self.scopes.iter().rev();
        for stack in stack_iter {
            match stack.values.get(name) {
                None => continue,
                Some(expr) => return Ok(expr),
            }
        }

        Err(RloxError::Interpret(RuntimeError::UndefinedVariable))
    }

    pub fn assign(&mut self, name: &str, res: LoxType) -> Result<()> {
        // return error if variable isn't defined.
        self.get(name)
            .map_err(|_| RloxError::Interpret(RuntimeError::UndefinedVariableAssignment))?;

        let stack_iter = self.scopes.iter_mut().rev();
        for stack in stack_iter {
            if stack.values.get(name).is_none() {
                continue;
            }
            stack.values.insert(name.to_string(), res);
            break;
        }

        Ok(())
    }

    // Return a mutable reference to the global environment
    pub fn define_globals(&mut self, name: &str, res: LoxType) {
        self.scopes[0].values.insert(name.to_string(), res);
    }

    pub fn get_globals(&mut self, name: &str) -> Result<&LoxType> {
        match self.scopes[0].values.get(name) {
            Some(expr) => Ok(expr),
            None => Err(RloxError::Interpret(RuntimeError::UndefinedVariable)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let mut env = Environment::default();

        env.define("var1", LoxType::Bool(true));
        env.define("var2", LoxType::Nil);

        assert_eq!(env.get("var2"), Ok(&LoxType::Nil));
        assert_eq!(env.get("var1"), Ok(&LoxType::Bool(true)));
    }

    #[test]
    fn test_assignment() {
        let mut env = Environment::default();

        env.define("var_test", LoxType::Bool(true));
        match env.assign("var_test", LoxType::Bool(false)) {
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

        env.define("fail_me", LoxType::Bool(true));

        // Fails because "var_test" was not previously defined
        match env.assign("var_test", LoxType::Bool(false)) {
            Ok(_) => (),
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    #[test]
    fn test_nested_get() {
        let mut root = Environment::default();
        root.define("name1", LoxType::Bool(true));
        root.define("name2", LoxType::Bool(false));
        root.new_scope();
        root.define("name3", LoxType::String("Found".to_string()));
        root.new_scope();

        if root.get("name1").is_err() {
            panic!("Nested environments did not work");
        }
        if root.get("name2").is_err() {
            panic!("Nested environments did not work");
        }

        if root.get("name3").is_err() {
            panic!("Nested environments did not work");
        }
    }

    #[test]
    fn test_nested_assignment() {
        let mut root = Environment::default();
        root.define("name1", LoxType::Bool(true));
        root.define("name2", LoxType::Bool(false));

        root.new_scope();
        root.define("name3", LoxType::String("Found".to_string()));

        root.new_scope();
        if root.assign("name4", LoxType::Number(32.)).is_ok() {
            panic!("Shouldn't assign to unknown variable.");
        }
        if root.assign("name1", LoxType::Nil).is_err() {
            panic!("Should assign to known variable in nested scope.");
        }

        root.drop();
        root.drop();

        if root.get("name1").unwrap() != &LoxType::Nil {
            panic!("Did not overwrite variable in parent scope.");
        }
    }

    #[test]
    fn dont_drop_globals() {
        // Ensure we can't accidentally drop the global environment

        let mut global = Environment::default();
        global.define("fun1", LoxType::Number(10.));

        // Create a bunch of environment and then drop them
        for _ in 1..10 {
            global.new_scope();
        }
        for _ in 1..5 {
            global.drop();
        }
        for _ in 1..10 {
            global.new_scope();
        }
        for _ in 1..20 {
            global.drop();
        }

        // Ensure final environment still contains our globals
        match global.get("fun1") {
            Ok(_) => (),
            Err(_) => panic!("Dropped the global environment"),
        }
    }
}
