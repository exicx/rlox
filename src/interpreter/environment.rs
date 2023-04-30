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

use std::collections::HashMap;

use crate::errors::{Result, RloxError, RuntimeError};

use super::LoxType;

/*
 * We need to keep a linked list of Environments, where later entires
 * are more local environments.
 * Each Environment contains the named declarations of a scope.
 * Global scope is the parent Environment of every chain.
 * When functions are executed, they need a new environment that only
 * has the global scope as a parent.
 *
 * So we'll have at least two things pointing to the global scope, each needing
 * an exclusive reference (&mut). For this we'll have to use Rc.
 *
 * From:
 *
 * struct Interpreter {
 *   env: Environment,
 * }
 *
 * struct Environment {
 *   scopes: Vec<Scope>,
 * }
 *
 * To:
 *
 * struct Interpreter {
 *   env: Environment,
 * }
 *
 * struct Environment {
 *   parent: Option<Box<Environment>>,
 *   values: HashMap<String, LoxType>,
 * }
 */

#[derive(Debug, Clone)]
struct Scope {
    values: HashMap<String, LoxType>,
}

impl Default for Scope {
    fn default() -> Self {
        let mut values = HashMap::new();
        values.reserve(4);
        Self { values }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Environment {
    scopes: Vec<Scope>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            scopes: vec![Scope::default()],
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    // Create a new scope at the end of the queue
    pub fn new_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    // Drop the top-most scope
    // self.scopes[0] is the global scope, don't allow dropping
    // that one.
    pub fn drop(&mut self) -> bool {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            true
        } else {
            // return false when all scopes are dropped (except for global)
            false
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let mut env = Environment::default();

        env.define("var1", LoxType::Bool(true));
        env.define("var2", LoxType::Nil);

        if let Ok(&LoxType::Nil) = env.get("var2") {
        } else {
            panic!("Mismatched types.");
        }

        if let Ok(&LoxType::Bool(true)) = env.get("var1") {
        } else {
            panic!("Mismatched types.");
        }
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

        if let Ok(&LoxType::Nil) = root.get("name1") {
        } else {
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
