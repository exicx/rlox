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

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    EOF,
}

#[derive(Debug)]
pub enum RloxError {
    Cmdline(String),
    Scan {
        line: usize,
        help: String,
        message: String,
    },
    Parse(ParseError),
    Interpret(String),
}

impl fmt::Display for RloxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}
impl error::Error for RloxError {}
