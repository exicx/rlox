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

use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, RloxError>;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    ParseFailure(String),
    UnexpectedToken(String),
    TooManyParameters,
    EOF,
}

#[derive(Debug, PartialEq)]
pub enum ResolverError {}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    TypeComparison(String),
    Concatenation(String),
    Arithmetic(String),
    UndefinedVariable, // Null access error
    UndefinedVariableAssignment,
    NotACallableType(String),
    MismatchedArguments(String),
}

#[derive(Debug, PartialEq)]
pub enum RloxError {
    Cmdline(String),
    Scan(ScanError),
    Parse(ParseError),
    Resolver(ResolverError),
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
                    "Scanning Error: [line {}:{}] {}:{}",
                    err.line, err.position, err.message, err.help
                )
            }
            Self::Parse(err) => {
                write!(f, "{err:?}")
            }
            Self::Resolver(err) => {
                write!(f, "{err:?}")
            }
            Self::Interpret(err) => {
                write!(f, "{err:?}")
            }
        }
    }
}
impl error::Error for RloxError {}
