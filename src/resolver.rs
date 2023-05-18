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

// Handle variable resolution. We care about the following AST nodes:
//
// 1) `stmt::block` statement introduces a new scope for the statements it contains.
// 2) `stmt::fun` declaration introduces a new scope for its body and binds its parameters in that scope.
// 3) `stmt::var` declaration adds a new variable to the current scope.
// 4) `expr::variable` and `expr::assign` expressions need to have their variables resolved.
//
//
// Scopes keeps track of how nested into the code we are. When scopes is len == 0, we're at global scope.
// The resolver does not concern itself with anything in the global scope.

// TODO:
// We're going to be adding a few extra things here:
// 1) Statements after `return` statements in a function.
//    Perhaps other forms of unreachable code.
// 2) `return` statements outside of any function.
//    It's not syntatically incorrect, it just doesn't make sense.
// 3) Variables used before defintion.
//    Lox will implicitly assign "nil" to undefined variables, but we're going to reject it here.

use std::collections::HashMap;

use crate::errors::{ResolverError, Result};
use crate::parser::ast::{Expr, Stmt};

#[derive(Default, Clone, Copy)]
enum FunctionType {
    #[default]
    None,
    Function,
}

#[derive(Default)]
pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolver(&mut self, program: &mut Vec<Stmt>) -> Result<()> {
        for stmt in program {
            self.resolve(stmt)?;
        }
        Ok(())
    }

    fn resolve(&mut self, stmt: &mut Stmt) -> Result<()> {
        log::trace!("{:?}", stmt);

        match stmt {
            Stmt::Block(body) => {
                log::trace!("block statement");

                // Block creates a new scope, we keep track of the number of hops
                // from the last environment
                //
                self.begin_scope();
                for stmt in body {
                    self.resolve(stmt)?;
                }
                self.end_scope();
            }
            Stmt::Expression(expr) => {
                self.resolve_expr(expr);
            }
            Stmt::Fun(ident, _, _) => {
                log::trace!("function statement");

                // Functions create a new scope, and also bind their params to names
                //
                self.declare(ident);
                self.define(ident);
                self.resolve_fun(stmt, FunctionType::Function);
            }
            Stmt::If(condition, if_body, else_body) => {
                log::trace!("if statement");
                self.resolve_expr(condition);
                self.resolve(if_body)?;

                if let Some(stmt) = else_body {
                    self.resolve(stmt)?;
                }
            }
            Stmt::Return(_, expr) => {
                log::trace!("return statement");

                if let FunctionType::None = self.current_function {
                    // Lint: don't allow return outside of functions
                    unimplemented!("return statement is not inside a function.")
                }

                if let Some(expr) = expr {
                    self.resolve_expr(expr);
                }
            }
            Stmt::Var(ident, init) => {
                log::trace!("variable declaration statement");

                // Variable declarations create new names
                //

                self.declare(ident);
                if let Some(expr) = init {
                    self.resolve_expr(expr);
                }
                self.define(ident);
            }
            Stmt::While(condition, body) => {
                log::trace!("while statement");
                self.resolve_expr(condition);
                self.resolve(body)?;
            }
        }

        Ok(())
    }

    fn resolve_fun(&mut self, stmt: &mut Stmt, ftype: FunctionType) {
        let saved_ftype = self.current_function;
        self.current_function = ftype;

        self.begin_scope();
        if let Stmt::Fun(_, params, body) = stmt {
            for param in params {
                self.declare(param.lexeme());
                self.define(param.lexeme());
            }

            // resolve function body
            self.resolver(body);
        } else {
            unreachable!();
        };
        self.end_scope();

        self.current_function = saved_ftype;
    }

    fn resolve_expr(&mut self, root: &mut Expr) {
        // Expr::Assign() and Expr::Variable() both access variable names for
        // assignment (write, l-value) and access (read, r-value) respectively.
        // But we descend into all expressions recursively to resolve any mentions.
        // E.g. Stmt::If can contain an Expr::Logical which contains Expr::Binary which
        // references the Expr::Variable "a".

        // We could also resolve Expr::Call(), but we don't do that yet because it gets more
        // complicated with method calls.

        match root {
            Expr::Assign(_, expr, _) => {
                log::trace!("assignment expression");

                self.resolve_expr(expr);
                self.resolve_local(root);
            }
            Expr::Binary(expr1, _, expr2) => {
                log::trace!("binary expression");

                self.resolve_expr(expr1);
                self.resolve_expr(expr2);
            }
            Expr::Call(expr, _, arguments) => {
                log::trace!("call expression");

                self.resolve_expr(expr);

                for i in arguments {
                    self.resolve_expr(i);
                }
            }
            Expr::Grouping(expr) => {
                log::trace!("grouping expression");

                self.resolve_expr(expr);
            }
            Expr::Literal(_) => {
                log::trace!("literal expression");

                // do nothing
            }
            Expr::Logical(expr1, _, expr2) => {
                log::trace!("logical expression");

                self.resolve_expr(expr1);
                self.resolve_expr(expr2);
            }
            Expr::Unary(_, expr) => {
                log::trace!("unary expression");

                self.resolve_expr(expr);
            }
            Expr::Variable(ident, _) => {
                log::trace!("variable expression");

                if !self.scopes.is_empty() {
                    match self.scopes.last().unwrap().get(ident) {
                        Some(false) => {
                            // Raise an error
                            // "Can't read local variable in its own initializer"
                            // return Err();
                            unimplemented!("can't read local variable in its own initializer");
                        }
                        None => (),
                        Some(_) => (),
                    }
                }

                self.resolve_local(root);
            }
        }
    }

    // Count the number of steps from the innermost scope
    // to where we find the variable name.
    // We send the expression, name, and number of hops to be stored in
    // the interpreter.
    fn resolve_local(&mut self, expr: &mut Expr) {
        assert!(matches!(expr, Expr::Assign(..) | Expr::Variable(..)));

        let name = match expr {
            Expr::Assign(name, ..) => name.clone(),
            Expr::Variable(name, ..) => name.clone(),
            _ => unreachable!(),
        };

        // If we find the variable in the list of scopes, then modify the expression
        // AST node = Some(distance).
        // Otherwise, we assume it's global and leave it None
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name) {
                match expr {
                    Expr::Assign(.., ref mut depth) => {
                        *depth = Some(distance as u32);
                    }
                    Expr::Variable(.., ref mut depth) => {
                        *depth = Some(distance as u32);
                    }
                    _ => unreachable!(),
                }
                break;
            }
        }

        log::trace!("expr: {:?}", expr)
    }

    // Create a name in the current scope (or do nothing if we're global)
    // It is an error to reference a variable in its own initializer.
    // We're explicitly preventing this code from being valid:
    //
    // var a = "outer";
    // {
    //   var a = a;
    // }
    fn declare(&mut self, ident: &str) {
        if self.scopes.is_empty() {
            // we're in global scope
            return;
        }

        let last = self.scopes.last_mut().unwrap();
        if last.get(ident).is_some() {
            // Lint: Don't allow re-declaring a variable in scopes
            // (global is fine)
            unimplemented!("variable re-declared")
        }

        // insert name and mark it as un-initialized
        last.insert(ident.to_string(), false);
    }

    fn define(&mut self, ident: &str) {
        if self.scopes.is_empty() {
            // we're in global scope
            return;
        }

        let last = self.scopes.last_mut().unwrap();
        // insert name and mark it as initialized
        last.insert(ident.to_string(), true);
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        todo!();
    }
}
