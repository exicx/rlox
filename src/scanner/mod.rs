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

mod input;
mod tokens;

use std::collections::HashMap;
use std::fmt::{self, Debug, Write};
use std::str::FromStr;

use self::input::{Input, InputIter, PeekableIterator};
use crate::errors::{Result, RloxError, ScanError};
use crate::parser::Parser;

pub use tokens::{Token, TokenLiteral, TokenType};

pub struct Scanner {
    tokens: Vec<Token>,
    keywords: HashMap<String, TokenType>,
}

impl Debug for Scanner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_out: String = String::new();
        debug_out.write_str("Scanner: \n").unwrap();

        for sc in &self.tokens {
            debug_out.write_fmt(format_args!("\t{sc}\n")).unwrap();
        }

        write!(f, "{debug_out}")
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

impl Scanner {
    pub fn new() -> Scanner {
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
            tokens: vec![],
            keywords,
        }
    }

    // TODO: Turn this whole scanning process into an iterator
    // I want to return an iterator to the caller that, when .next()'d,
    // returns the next token from the source.
    // 1 Create scanner
    // 2 Pass source to scanner
    // 3 Iterate over scanner
    pub fn scan_tokens(&mut self, source: &str) -> Result<()> {
        // Create our custom iterator from the input string
        let mut input_iter = Input::new(source).into_iter();

        // Loop over each grapheme and try to scan it into tokens
        while let Some(item) = input_iter.next() {
            let mut lexeme = String::from_str(item).expect("tried making a string");
            let start_line = input_iter.line();
            let start_pos = input_iter.location();

            let token = match item {
                // Single character tokens
                "(" => Some(TokenType::LeftParen),
                ")" => Some(TokenType::RightParen),
                "{" => Some(TokenType::LeftBrace),
                "}" => Some(TokenType::RightBrace),
                "," => Some(TokenType::Comma),
                "." => Some(TokenType::Dot),
                "-" => Some(TokenType::Minus),
                "+" => Some(TokenType::Plus),
                ";" => Some(TokenType::Semicolon),
                "/" => {
                    if match_next_char((&mut input_iter, &mut lexeme), "/") {
                        scan_forward_until((&mut input_iter, &mut lexeme), "\n");
                        None // Comment (dropped by scanner)
                    } else {
                        Some(TokenType::Slash) // /
                    }
                }
                "*" => Some(TokenType::Star),

                // One or two character tokens
                "!" => {
                    if match_next_char((&mut input_iter, &mut lexeme), "=") {
                        Some(TokenType::BangEqual) // !=
                    } else {
                        Some(TokenType::Bang) // =
                    }
                }
                "=" => {
                    if match_next_char((&mut input_iter, &mut lexeme), "=") {
                        Some(TokenType::EqualEqual) // ==
                    } else {
                        Some(TokenType::Equal) // =
                    }
                }
                ">" => {
                    if match_next_char((&mut input_iter, &mut lexeme), "=") {
                        Some(TokenType::GreaterEqual) // >=
                    } else {
                        Some(TokenType::Greater) // >
                    }
                }
                "<" => {
                    if match_next_char((&mut input_iter, &mut lexeme), "=") {
                        Some(TokenType::LessEqual) // <=
                    } else {
                        Some(TokenType::Less) // <
                    }
                }

                // Whitespace
                ch if is_whitespace(ch) => None, // Whitespace (dropped by scanner)

                // Literals
                // Identifiers and reserved keywords
                ch if is_lowercase(ch) | is_uppercase(ch) | (ch == "_") => {
                    // Identifiers can begin with a-z, A-Z, or _.
                    // But they can contain a-z, A-Z, 0-9, or _.
                    while is_alpha_numeric(input_iter.peek()) {
                        lexeme.push_str(input_iter.next().unwrap());
                    }

                    // If the identifier exists in our hashmap of keywords, then treat it like a keyword
                    if let Some(token) = self.keywords.get(&lexeme) {
                        Some(*token)
                    } else {
                        // Otherwise it's just an identifier
                        Some(TokenType::Identifier)
                    }
                }
                // Strings
                "\"" => {
                    string((&mut input_iter, &mut lexeme))?;
                    Some(TokenType::String)
                }
                // Numbers
                ch if is_digit(ch) => {
                    number((&mut input_iter, &mut lexeme))?;
                    Some(TokenType::Number)
                }
                x => {
                    return Err(RloxError::Scan(ScanError::new(
                        start_line,
                        start_pos,
                        x,
                        "Character not recognized",
                    )))
                }
            };

            // Add scanned token to list of tokens
            match token {
                None => (),
                Some(token_type) => {
                    self.tokens
                        .push(Token::new(token_type, lexeme, start_line, start_pos));
                }
            }
        }

        // Add an EOF token at end of input
        self.tokens
            .push(Token::new(TokenType::Eof, "".into(), input_iter.line(), 0));

        Ok(())
    }

    pub fn into_parser(self) -> Parser {
        Parser::new(self.tokens)
    }
}

/// Helper functions
/// These all take a ScannerState struct to build up
/// the scanned lexeme.
///
fn is_whitespace(ch: &str) -> bool {
    matches!(ch, " " | "\n" | "\t" | "\r")
}

fn is_digit(ch: &str) -> bool {
    matches!(
        ch,
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
    )
}

fn is_lowercase(ch: &str) -> bool {
    matches!(
        ch,
        "a" | "b"
            | "c"
            | "d"
            | "e"
            | "f"
            | "g"
            | "h"
            | "i"
            | "j"
            | "k"
            | "l"
            | "m"
            | "n"
            | "o"
            | "p"
            | "q"
            | "r"
            | "s"
            | "t"
            | "u"
            | "v"
            | "w"
            | "x"
            | "y"
            | "z"
    )
}

fn is_uppercase(ch: &str) -> bool {
    matches!(
        ch,
        "A" | "B"
            | "C"
            | "D"
            | "E"
            | "F"
            | "G"
            | "H"
            | "I"
            | "J"
            | "K"
            | "L"
            | "M"
            | "N"
            | "O"
            | "P"
            | "Q"
            | "R"
            | "S"
            | "T"
            | "U"
            | "V"
            | "W"
            | "X"
            | "Y"
            | "Z"
    )
}

// a-z, A-Z, 0-9, _
fn is_alpha_numeric(ch: Option<&str>) -> bool {
    let ch = match ch {
        None => return false,
        Some(char) => char,
    };

    match ch {
        _ if is_lowercase(ch) => true,
        _ if is_uppercase(ch) => true,
        _ if is_digit(ch) => true,
        "_" => true,
        _ => false,
    }
}

fn scan_forward_until((iter, lexeme): (&mut InputIter, &mut String), ch: &str) {
    // Add each scanned grapheme until we hit the stopping condition
    iter.take_while(|ic| *ic != ch)
        .for_each(|x| lexeme.push_str(x));
}

fn match_next_char((iter, lexeme): (&mut InputIter, &mut String), ch: &str) -> bool {
    if iter.peek() != Some(ch) {
        false
    } else {
        let grapheme = iter.next().unwrap();
        lexeme.push_str(grapheme);
        true
    }
}

fn string((iter, lexeme): (&mut InputIter, &mut String)) -> Result<()> {
    // Scan till we find another double quote.
    // Lox doesn't support escaped quotes, so this is simple.
    scan_forward_until((iter, lexeme), "\"");

    // Got to end of file without a terminating string
    if iter.peek().is_none() {
        return Err(RloxError::Scan(ScanError::new(
            iter.line(),
            iter.location(),
            "",
            "Unterminated string",
        )));
    }

    Ok(())
}

fn number((iter, lexeme): (&mut InputIter, &mut String)) -> Result<()> {
    for item in iter.by_ref() {
        if is_digit(item) || item == "." {
            lexeme.push_str(item);
        } else {
            break;
        }
    }

    // If the scanned number has more than one decimal, then we fail
    if lexeme.chars().filter(|ch| *ch == '.').count() > 1 {
        return Err(RloxError::Scan(ScanError::new(
            iter.line(),
            iter.location(),
            &lexeme,
            "Number contained two or more decimals",
        )));
    }

    // A number ending in a decimal is also an error
    // Once scanning the number is complete, check if the last char
    // is a decimal.
    if lexeme.ends_with('.') {
        return Err(RloxError::Scan(ScanError::new(
            iter.line(),
            iter.location(),
            &lexeme,
            "No digits after decimal",
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokens::*;

    fn setup_scanner1() -> Scanner {
        let input = r#"
            // this is a comment
            var andy = 10;
            var jonny = 3;
            if (andy and jonny) { print "Hello World" + (andy+jonny) };
            "#;

        let mut scanner = Scanner::new();
        scanner.scan_tokens(input).unwrap();
        scanner
    }

    #[test]
    fn test_and_token() {
        let input = "var andy = 20; if (andy or 0) { print \"fail\"; }";
        let mut scanner = Scanner::new();
        scanner.scan_tokens(input).unwrap();

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
