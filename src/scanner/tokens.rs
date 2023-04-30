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
    Identifier(String),
    None,
}

impl Display for TokenLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLiteral::Str(lit_str) => write!(f, "{lit_str}"),
            TokenLiteral::None => write!(f, "None"),
            TokenLiteral::Number(num) => write!(f, "{num}"),
            TokenLiteral::Identifier(iden) => write!(f, "{iden}"),
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
    pub fn new(token_type: TokenType, lexeme: String, line: usize, position: usize) -> Token {
        let tl = match token_type {
            TokenType::String => {
                let string_literal = lexeme.trim_matches('"').into();
                TokenLiteral::Str(string_literal)
            }
            TokenType::Number => {
                // TODO fix the unwrap() here. Add new error type for token generation failures
                let number = lexeme.parse::<f64>().unwrap();
                TokenLiteral::Number(number)
            }
            TokenType::Identifier => {
                let identifier = lexeme.clone();
                TokenLiteral::Identifier(identifier)
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

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} on line {}:{}",
            self.token_type, self.lexeme, self.line, self.position
        )
    }
}
