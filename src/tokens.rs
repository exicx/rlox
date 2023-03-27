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

use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier, // var X;
    String,     // "string"
    Number,     // 0.123
    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun, // function
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenLiteral {
    Str(String),
    Number(f64),
    None,
}

impl Display for TokenLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLiteral::Str(lit_str) => write!(f, "{lit_str}"),
            TokenLiteral::None => write!(f, "None"),
            TokenLiteral::Number(num) => write!(f, "{num}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: TokenLiteral, // parsed value of lexeme
    line: usize,           // line number where token was found
    position: usize,       // character position within file
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &[char], line: usize, position: usize) -> Token {
        let lexeme: String = lexeme.iter().collect();
        let tl = match token_type {
            TokenType::String => {
                let string_literal = lexeme.trim_matches('"').into();
                TokenLiteral::Str(string_literal)
            }
            TokenType::Number => {
                // TODO fix the unwrap() here. Add new error type for token generation failures
                TokenLiteral::Number(lexeme.parse::<f64>().unwrap())
            }
            _ => TokenLiteral::None,
        };
        Token {
            token_type,
            lexeme,
            literal: tl,
            line,
            position,
        }
    }

    pub fn token_type(&self) -> TokenType {
        self.token_type
    }

    pub fn token_literal(&self) -> &TokenLiteral {
        &self.literal
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} on line {} at position {}",
            self.token_type, self.lexeme, self.line, self.position
        )
    }
}
