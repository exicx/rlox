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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::errors::{Result, RloxError, RuntimeError};

use super::LoxType;

pub type RfEnv = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
pub struct Environment {
    parent: Option<RfEnv>,
    env: HashMap<String, LoxType>,
}

pub fn new_global() -> RfEnv {
    Rc::new(RefCell::new(Environment {
        parent: None,
        env: HashMap::new(),
    }))
}

pub fn from(env: &RfEnv) -> RfEnv {
    log::trace!("from");
    Rc::new(RefCell::new(Environment {
        parent: Some(Rc::clone(env)),
        env: HashMap::new(),
    }))
}

// Drop the top-most scope, but never global
pub fn drop(rfenv: &RfEnv) -> RfEnv {
    log::trace!("dropping");

    match rfenv.borrow().parent {
        None => Rc::clone(rfenv),
        Some(ref parent) => Rc::clone(parent),
    }
}

fn ancestor(rfenv: RfEnv, depth: u32) -> RfEnv {
    if depth == 0 {
        return Rc::clone(&rfenv);
    }

    let binding = rfenv.borrow();
    let parent = match &binding.parent {
        Some(parent) => parent,
        None => {
            panic!("depth is >1, but we're already at the root env");
        }
    };
    ancestor(Rc::clone(parent), depth - 1)
}

// Define a new type
pub fn define(env: &RfEnv, key: &str, val: LoxType) {
    log::trace!("defining: {}", key);
    env.borrow_mut().env.insert(key.to_string(), val);
}

pub fn get_n(rfenv: &RfEnv, key: &str, depth: u32) -> Result<LoxType> {
    log::trace!("fast get: {}", key);

    let parent = ancestor(Rc::clone(rfenv), depth);
    assert!(parent.borrow().env.contains_key(key));

    let binding = parent.borrow();
    match binding.env.get(key) {
        None => unreachable!(),
        Some(val) => Ok(val.clone()),
    }
}

// Return value if it exists, otherwise error
// Recurses up call stack
pub fn get(rfenv: &RfEnv, key: &str) -> Result<LoxType> {
    log::trace!("slow getting: {}", key);

    match rfenv.borrow().env.get(key) {
        None => {
            log::trace!("  {} not found in this environment.", key);
            match rfenv.borrow().parent {
                None => Err(RloxError::Interpret(RuntimeError::UndefinedVariable)),
                Some(ref parent) => get(parent, key),
            }
        }
        Some(val) => Ok(val.clone()),
    }
}

pub fn assign_n(rfenv: &RfEnv, key: &str, val: LoxType, depth: u32) -> Result<()> {
    log::trace!("fast assign: {} -> {}", key, val);

    let parent = ancestor(Rc::clone(rfenv), depth);

    get(&parent, key)
        .map_err(|_| RloxError::Interpret(RuntimeError::UndefinedVariableAssignment))?;

    parent.borrow_mut().env.insert(key.to_string(), val);

    Ok(())
}

pub fn assign(rfenv: &RfEnv, key: &str, val: LoxType) -> Result<()> {
    log::trace!("slow assigning: {} -> {}", key, val);

    // return error if variable isn't defined.
    get(rfenv, key).map_err(|_| RloxError::Interpret(RuntimeError::UndefinedVariableAssignment))?;

    // we know the assignment exists somewhere, so now we just need to find it.
    // check current scope

    if rfenv.borrow().env.get(key).is_none() {
        match &rfenv.borrow().parent {
            Some(parent) => return assign(parent, key, val),
            _ => unreachable!(),
        }
    } else {
        rfenv.borrow_mut().env.insert(key.to_string(), val);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basics() {
        let env = new_global();

        define(&env, "var1", LoxType::Bool(true));
        define(&env, "var2", LoxType::Nil);

        if let Ok(LoxType::Nil) = get(&env, "var2") {
        } else {
            panic!("Mismatched types.");
        }

        if let Ok(LoxType::Bool(true)) = get(&env, "var1") {
        } else {
            panic!("Mismatched types.");
        }
    }

    #[test]
    fn test_assignment() {
        let env = new_global();

        define(&env, "var_test", LoxType::Bool(true));

        match assign(&env, "var_test", LoxType::Bool(false)) {
            Ok(_) => (),
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    #[test]
    #[should_panic]
    fn test_failing_assignment() {
        let env = new_global();

        define(&env, "fail_me", LoxType::Bool(true));

        // Fails because "var_test" was not previously defined
        match assign(&env, "var_test", LoxType::Bool(false)) {
            Ok(_) => (),
            Err(e) => {
                panic!("{e}");
            }
        }
    }

    #[test]
    fn test_nested_get() {
        let root = new_global();
        define(&root, "name1", LoxType::Bool(true));
        define(&root, "name2", LoxType::Bool(false));

        let env1 = from(&root);
        define(&root, "name3", LoxType::String("Found".to_string()));

        let env2 = from(&root);

        if get(&root, "name1").is_err() {
            panic!("Nested environments did not work");
        }
        if get(&root, "name2").is_err() {
            panic!("Nested environments did not work");
        }

        if get(&root, "name3").is_err() {
            panic!("Nested environments did not work");
        }
    }

    #[test]
    fn test_simple_assignment() {
        let root = new_global();
        let env1 = from(&root);

        define(&root, "name1", LoxType::Nil);
        define(&env1, "name2", LoxType::Nil);
        if let Ok(LoxType::Nil) = get(&env1, "name1") {
        } else {
            panic!("name was not defined.");
        }

        assign(&env1, "name1", LoxType::Bool(true));
        if let Ok(LoxType::Bool(true)) = get(&env1, "name1") {
        } else {
            panic!("name was not defined.");
        }
    }

    #[test]
    fn test_nested_assignment() {
        let root = new_global();
        let env1 = from(&root);
        let env2 = from(&env1);

        define(&root, "name1", LoxType::Bool(true));
        define(&root, "name2", LoxType::Bool(false));
        define(&env1, "name3", LoxType::String("Found".to_string()));

        if assign(&env2, "name4", LoxType::Number(32.)).is_ok() {
            panic!("Shouldn't assign to unknown variable.");
        }
        if assign(&env2, "name1", LoxType::Nil).is_err() {
            panic!("Should assign to known variable in nested scope.");
        }

        if let Ok(LoxType::Nil) = get(&root, "name1") {
        } else {
            panic!("Did not overwrite variable in parent scope.");
        }
    }

    #[test]
    fn dont_drop_globals() {
        // Ensure we can't accidentally drop the global environment

        let mut root = new_global();
        define(&root, "fun1", LoxType::Number(10.));
        let global = Rc::clone(&root);

        // Create a bunch of environments and then drop them
        for _ in 1..10 {
            root = from(&root);
        }
        for _ in 1..5 {
            root = drop(&root);
        }
        for _ in 1..10 {
            root = from(&root);
        }
        for _ in 1..20 {
            root = drop(&root);
        }

        // Ensure final environment still contains our globals
        match get(&global, "fun1") {
            Ok(_) => (),
            Err(_) => panic!("Dropped the global environment"),
        }
    }
}
