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
pub struct ScanError {
    line: usize,
    position: usize,
    help: String,
    message: String,
}

impl ScanError {
    pub fn new(line: usize, position: usize, help: &str, message: &str) -> Self {
        Self {
            line,
            position,
            help: help.to_string(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    ParseFailure,
    UnexpectedToken(String),
    EOF,
}

#[derive(Debug)]
pub enum RuntimeError {
    TypeComparison(String),
    Concatenation(String),
    Math(String),
}

#[derive(Debug)]
pub enum RloxError {
    Cmdline(String),
    Scan(ScanError),
    Parse(ParseError),
    Interpret(RuntimeError),
}

impl fmt::Display for RloxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cmdline(str) => {
                write!(f, "{str}")
            }
            Self::Scan(err) => {
                write!(
                    f,
                    "[line {}:{}] {}:{}",
                    err.line, err.position, err.message, err.help
                )
            }
            Self::Parse(err) => {
                write!(f, "{err:?}")
            }
            Self::Interpret(err) => {
                write!(f, "{err:?}")
            }
        }
    }
}
impl error::Error for RloxError {}
