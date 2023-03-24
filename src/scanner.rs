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

use std::collections::HashMap;
use std::fmt::{self, Write};

use crate::errors;
use crate::parser::Parser;
use crate::tokens::{Token, TokenType};

// TODO: We can turn the Scanner into an iterator so Parser can operate on it.
// TODO: We're not using line numbers as much as we could be for errors.
// TODO: We're not really using errors very well either.
// Nevertheless, the scanner is 'complete'.
pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
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
        keywords.insert("print".into(), TokenType::Print);
        keywords.insert("return".into(), TokenType::Return);
        keywords.insert("super".into(), TokenType::Super);
        keywords.insert("this".into(), TokenType::This);
        keywords.insert("true".into(), TokenType::True);
        keywords.insert("var".into(), TokenType::Var);
        keywords.insert("while".into(), TokenType::While);
        keywords.insert("eof".into(), TokenType::Eof);

        Self {
            source: Vec::new(),
            tokens: vec![],
            line: 1,
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
        Scanner {
            source: source.chars().collect(),
            ..Default::default()
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), errors::RloxError> {
        while !self.is_at_end() {
            self.start = self.current;
            let res = self.scan_token();

            // Currently strings, numbers, and identifiers can add their own tokens
            // Fix this in the future.
            // Those tokens will return None from scan_token() so they're not added twice.
            if let Some(token) = res {
                self.add_token(&token);
            }
        }

        self.tokens.push(Token::new(TokenType::Eof, &[], self.line));

        Ok(())
    }

    fn scan_token(&mut self) -> Option<TokenType> {
        let ch = self.advance()?;

        let token: Option<TokenType> = match *ch {
            // Single character tokens
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
            ',' => Some(TokenType::Comma),
            '.' => Some(TokenType::Dot),
            '-' => Some(TokenType::Minus),
            '+' => Some(TokenType::Plus),
            ';' => Some(TokenType::Semicolon),
            '/' => {
                if self.match_next_char(&'/') {
                    // Read ahead to end of line
                    while self.peek() != Some(&'\n') && !self.is_at_end() {
                        self.advance();
                    }
                    None
                } else {
                    Some(TokenType::Slash)
                }
            }
            '*' => Some(TokenType::Star),

            // One or two character tokens
            '!' => {
                if self.match_next_char(&'=') {
                    Some(TokenType::BangEqual)
                } else {
                    Some(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_next_char(&'=') {
                    Some(TokenType::EqualEqual)
                } else {
                    Some(TokenType::Equal)
                }
            }
            '>' => {
                if self.match_next_char(&'=') {
                    Some(TokenType::GreaterEqual)
                } else {
                    Some(TokenType::Greater)
                }
            }
            '<' => {
                if self.match_next_char(&'=') {
                    Some(TokenType::LessEqual)
                } else {
                    Some(TokenType::Less)
                }
            }

            // Whitespace
            ' ' => None,
            '\n' => {
                self.line += 1;
                None
            }
            '\t' => None,
            '\r' => None,

            // Literals
            // Identifiers and reserved keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let result = self.identifier();
                match result {
                    Ok(_) => None,
                    Err(err) => {
                        // TODO: This is weird. Fix this.
                        eprintln!("{err}");
                        None
                    }
                }
            }
            // Strings
            '"' => {
                let result = self.string();
                match result {
                    Ok(_) => None,
                    Err(err) => {
                        // TODO: This is weird. Fix this.
                        eprintln!("{err}");
                        None
                    }
                }
            }
            // Numbers
            '0'..='9' => {
                let result = self.number();
                match result {
                    Ok(_) => None,
                    Err(err) => {
                        // TODO: This is weird. Fix this.
                        eprintln!("[line {}] {}", self.line, err);
                        None
                    }
                }
            }
            _x => {
                // TODO: Proper error type and handling here.
                println!("[line {}] Token not supported. {}.", self.line, _x);
                None
            }
        };

        token
    }

    // Helper functions

    // TODO: XXX: Replace these with iterator
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> Option<&char> {
        self.source.get(self.current)
    }

    // problem here. we're advancing current before getting it
    fn advance(&mut self) -> Option<&char> {
        if self.peek().is_some() {
            self.current += 1;
        }
        self.peek()
    }

    fn match_next_char(&mut self, ch: &char) -> bool {
        if self.peek() != Some(ch) {
            return false;
        }
        self.advance();
        true
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != Some(&'"') && !self.is_at_end() {
            if self.peek() == Some(&'\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            // TODO: Error type here
            return Err("Unterminated String".into());
        }

        self.advance(); // consume ending quote

        // Trim surrounding quotes of string literal.
        // self.tokens.push(Token::new(
        //     TokenType::String,
        //     &self.source[(self.start + 1)..(self.current - 1)],
        //     self.line,
        // ));
        self.add_token(&TokenType::String);

        Ok(())
    }

    fn number(&mut self) -> Result<(), String> {
        let mut decimal = false;
        let mut res: Result<(), String> = Ok(());

        while self.peek_is_digit().is_some() && !self.is_at_end() {
            match self.peek().unwrap() {
                '0'..='9' => {}
                '.' => {
                    // we can only have 1 decimal point in a number
                    if decimal {
                        res = Err("Too many decimals".into());
                    }
                    decimal = true;
                    self.advance();
                    match self.peek().unwrap() {
                        '0'..='9' => continue,
                        _ => {
                            res = Err("No digits after decimal".into());
                            break;
                        }
                    }
                }
                _ => {}
            }
            self.advance();
        }

        // return error if any
        res.as_ref()?;

        self.add_token(&TokenType::Number);

        Ok(())
    }

    fn add_token(&mut self, token: &TokenType) {
        self.tokens.push(Token::new(
            *token,
            &self.source[self.start..self.current],
            self.line,
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
    fn is_alpha_numeric(&self, ch: Option<&char>) -> bool {
        if ch.is_none() {
            return false;
        }
        let ch = ch.unwrap();
        ch.is_alphanumeric() || ch == &'_'
    }

    fn identifier(&mut self) -> Result<(), String> {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let substring: String = self.source[self.start..self.current].iter().collect();
        let keyword = self.keywords.get(&substring);

        // If the identifier exists in our hashmap of keywords, then treat it like a keyword
        if let Some(token) = keyword {
            let token = *token;
            self.add_token(&token);
        } else {
            // Otherwise it's an identifier and we lex it as such.
            self.add_token(&TokenType::Identifier);
        }

        Ok(())
    }

    pub fn into_parser(self) -> Parser {
        Parser::new(self.tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::*;

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
            if token.token_type() == &TokenType::And {
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
