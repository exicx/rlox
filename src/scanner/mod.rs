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

mod tokens;

use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::errors::{Result, RloxError, ScanError};
use crate::parser::Parser;
pub use tokens::{Token, TokenLiteral, TokenType};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    position: usize,
    keywords: HashMap<String, TokenType>,
}

impl Default for Scanner {
    fn default() -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("and".into(), TokenType::And);
        keywords.insert("class".into(), TokenType::Class);
        keywords.insert("else".into(), TokenType::Else);
        keywords.insert("false".into(), TokenType::False);
        keywords.insert("fun".into(), TokenType::Fun);
        keywords.insert("for".into(), TokenType::For);
        keywords.insert("if".into(), TokenType::If);
        keywords.insert("nil".into(), TokenType::Nil);
        keywords.insert("or".into(), TokenType::Or);
        // we can remove this if we break compatibility with Lox
        keywords.insert("print".into(), TokenType::Print);
        keywords.insert("return".into(), TokenType::Return);
        keywords.insert("super".into(), TokenType::Super);
        keywords.insert("this".into(), TokenType::This);
        keywords.insert("true".into(), TokenType::True);
        keywords.insert("var".into(), TokenType::Var);
        keywords.insert("while".into(), TokenType::While);
        keywords.insert("eof".into(), TokenType::Eof);

        Self {
            source: vec![],
            tokens: vec![],
            line: 1,
            position: 1,
            start: 0,
            current: 0,
            keywords,
        }
    }
}

impl fmt::Debug for Scanner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_out: String = String::new();
        debug_out.write_str("Scanner: \n").unwrap();

        for sc in &self.tokens {
            debug_out.write_fmt(format_args!("\t{sc}\n")).unwrap();
        }

        write!(f, "{debug_out}")
    }
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        let source: Vec<char> = source.chars().collect();
        Scanner {
            source,
            ..Default::default()
        }
    }

    pub fn scan_tokens(&mut self) -> Result<()> {
        while !self.is_at_end() {
            // mark the begining of scanning for
            // multi-char tokens
            self.start = self.current;

            if let Some(token_type) = self.scan_token()? {
                self.add_token(token_type)
            }
        }

        // Add an EOF token at end of input
        self.tokens
            .push(Token::new(TokenType::Eof, &[], self.line, 0));

        Ok(())
    }

    fn scan_token(&mut self) -> Result<Option<TokenType>> {
        let ch = self.advance();
        if ch.is_none() {
            // Non-fatal error, but there's no more tokens.
            return Ok(None);
        }
        let ch = ch.unwrap();

        match ch {
            // Single character tokens
            '(' => Ok(Some(TokenType::LeftParen)),
            ')' => Ok(Some(TokenType::RightParen)),
            '{' => Ok(Some(TokenType::LeftBrace)),
            '}' => Ok(Some(TokenType::RightBrace)),
            ',' => Ok(Some(TokenType::Comma)),
            '.' => Ok(Some(TokenType::Dot)),
            '-' => Ok(Some(TokenType::Minus)),
            '+' => Ok(Some(TokenType::Plus)),
            ';' => Ok(Some(TokenType::Semicolon)),
            '/' => {
                if self.match_next_char('/') {
                    // Read ahead to end of line
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                    Ok(None)
                } else {
                    Ok(Some(TokenType::Slash))
                }
            }
            '*' => Ok(Some(TokenType::Star)),

            // One or two character tokens
            '!' => {
                if self.match_next_char('=') {
                    Ok(Some(TokenType::BangEqual))
                } else {
                    Ok(Some(TokenType::Bang))
                }
            }
            '=' => {
                if self.match_next_char('=') {
                    Ok(Some(TokenType::EqualEqual))
                } else {
                    Ok(Some(TokenType::Equal))
                }
            }
            '>' => {
                if self.match_next_char('=') {
                    Ok(Some(TokenType::GreaterEqual))
                } else {
                    Ok(Some(TokenType::Greater))
                }
            }
            '<' => {
                if self.match_next_char('=') {
                    Ok(Some(TokenType::LessEqual))
                } else {
                    Ok(Some(TokenType::Less))
                }
            }

            // Whitespace
            ' ' | '\n' | '\t' | '\r' => Ok(None),

            // Literals
            // Identifiers and reserved keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                while self.is_alpha_numeric(self.peek()) {
                    self.advance();
                }
                let substring: String = self.source[self.start..self.current].iter().collect();
                let keyword = self.keywords.get(&substring);

                // If the identifier exists in our hashmap of keywords, then treat it like a keyword
                if let Some(token) = keyword {
                    Ok(Some(*token))
                } else {
                    // Otherwise it's an identifier and we lex it as such.
                    Ok(Some(TokenType::Identifier))
                }
            }
            // Strings
            '"' => {
                self.string()?;
                Ok(Some(TokenType::String))
            }
            // Numbers
            '0'..='9' => {
                self.number()?;
                Ok(Some(TokenType::Number))
            }
            x => Err(RloxError::Scan(ScanError::new(
                self.line,
                self.position - 1,
                &format!("{}", x),
                "Character not supported",
            ))),
        }
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != Some('"') && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            return Err(RloxError::Scan(ScanError::new(
                self.line,
                self.position,
                "",
                "Unterminated string",
            )));
        }

        self.advance(); // consume ending quote

        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        let mut decimal = false;

        while self.peek_is_digit().is_some() && !self.is_at_end() {
            match self.peek().unwrap() {
                '0'..='9' => {}
                '.' => {
                    if decimal {
                        // we can only have 1 decimal point in a number
                        return Err(RloxError::Scan(ScanError::new(
                            self.line,
                            self.position,
                            "",
                            "Number contained two or more decimals.",
                        )));
                    } else {
                        decimal = true;
                        self.advance();
                        match self.peek().unwrap() {
                            '0'..='9' => continue,
                            _ => {
                                return Err(RloxError::Scan(ScanError::new(
                                    self.line,
                                    self.position,
                                    "",
                                    "No digits after decimal",
                                )));
                            }
                        }
                    }
                }
                _ => {
                    unimplemented!("Unreachable.")
                }
            }
            self.advance();
        }

        Ok(())
    }

    // Helper functions
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.current += 1;
            self.position += 1; // position in line
        }

        if ch == Some('\n') {
            self.line += 1; // increment line on newline
            self.position = 1; // reset line position on newline
        }

        ch
    }

    fn match_next_char(&mut self, ch: char) -> bool {
        if self.peek() != Some(ch) {
            false
        } else {
            self.advance();
            true
        }
    }

    fn add_token(&mut self, token: TokenType) {
        self.tokens.push(Token::new(
            token,
            &self.source[self.start..self.current],
            self.line,
            self.position - (self.current - self.start),
        ));
    }

    fn peek_is_digit(&self) -> Option<bool> {
        let ch = self.peek()?;
        match ch {
            '0'..='9' => Some(true),
            '.' => Some(true),
            _ => None,
        }
    }

    // a-z, A-Z, 0-9, _
    // This also accepts unicode alphanumeric code points, but that's okay for us for now.
    fn is_alpha_numeric(&self, ch: Option<char>) -> bool {
        if ch.is_none() {
            return false;
        }
        let ch = ch.unwrap();
        ch.is_alphanumeric() || ch == '_'
    }

    pub fn into_parser(self) -> Parser {
        Parser::new(self.tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokens::*;

    fn setup_scanner1() -> Scanner {
        let mut scanner = Scanner::new(
            r#"
            // this is a comment
            var andy = 10;
            var jonny = 3;
            if (andy and jonny) { print "Hello World" + (andy+jonny) };
            "#,
        );
        scanner.scan_tokens().unwrap();
        scanner
    }

    #[test]
    fn test_and_token() {
        let mut scanner = Scanner::new("var andy = 20; if (andy or 0) { print \"fail\"; }");
        scanner.scan_tokens().unwrap();

        for token in scanner.tokens {
            if token.token_type() == TokenType::And {
                panic!("Unexpected \"AND\" token in program.");
            }
        }
    }

    #[test]
    fn test_scanner() {
        let scanner = setup_scanner1();
        let _parser = scanner.into_parser();
    }
}
