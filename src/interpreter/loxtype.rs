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

use super::callable::{FfiClock, FfiPrint, LoxFunction};
use std::fmt;

// TODO: getting rid of Clone here would allow using trait objects
// as the type for Functions, Classes, and Native FFI
#[derive(Debug, Clone)]
pub enum LoxType {
    Bool(bool),
    Number(f64),
    String(String),
    Fun(LoxFunction),
    Clock(FfiClock), // Make this more generic so we can define more native FFI
    Print(FfiPrint),
    // Fun(Box<dyn Callable>),
    // Class(Box<dyn Callable>),
    // Ffi(Box<dyn Callable>),
    Nil,
}

impl fmt::Display for LoxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxType::Bool(v) => {
                if *v {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            LoxType::Nil => write!(f, "nil"),
            LoxType::Number(n) => write!(f, "{}", n),
            LoxType::String(s) => write!(f, "{s}"),
            LoxType::Fun(call) => write!(f, "{call}"),
            LoxType::Clock(call) => write!(f, "{call}"),
            LoxType::Print(call) => write!(f, "{call}"),
            // LoxType::Class(call) => write!(f, "{call}"),
        }
    }
}
