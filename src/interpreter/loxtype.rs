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
