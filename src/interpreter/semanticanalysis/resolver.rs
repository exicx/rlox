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

use crate::parser::ast::{Expr, ExprLiteral, Stmt};

pub struct Resolver {
    program: Option<Vec<Stmt>>,
}

impl Resolver {
    pub fn new(program: Vec<Stmt>) -> Self {
        Self {
            program: Some(program),
        }
    }

    pub fn resolve(&mut self) -> Vec<Stmt> {
        if self.program.is_none() {
            return vec![];
        }

        for stmt in self.program.take().unwrap() {}

        self.program.take().unwrap()
    }
}
