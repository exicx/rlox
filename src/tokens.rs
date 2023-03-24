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
    Identifier,
    String,
    Number,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: TokenLiteral,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &[char], line: usize) -> Token {
        let mut tl: TokenLiteral = TokenLiteral::None;
        match token_type {
            TokenType::String => {
                let mut string_literal: String = lexeme.iter().collect();
                string_literal = string_literal.trim_matches('"').into();
                tl = TokenLiteral::Str(string_literal);
            }
            TokenType::Number => {
                tl = TokenLiteral::Number(lexeme.iter().collect::<String>().parse::<f64>().unwrap())
            }
            _ => {}
        }
        Token {
            token_type,
            lexeme: lexeme.iter().collect(),
            literal: tl,
            line,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn token_literal(&self) -> &TokenLiteral {
        &self.literal
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.token_type, self.lexeme)
    }
}
