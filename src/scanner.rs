use crate::errors;
use crate::tokens::{Token, TokenType};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    has_error: bool,
}

impl Scanner {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source.chars().collect(),
            tokens: vec![],
            line: 1,
            start: 0,
            current: 0,
            has_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), errors::RloxError> {
        while !self.is_at_end() {
            self.start = self.current;
            let res = self.scan_token();

            match res {
                Some(token) => {
                    self.tokens.push(Token::new(
                        token,
                        &self.source[self.start..self.current],
                        self.line,
                    ));
                }
                _ => {}
            }
        }

        self.tokens.push(Token::new(TokenType::Eof, &[], self.line));

        for token in &self.tokens {
            println!("{:?}", token);
        }

        Ok(())
    }

    fn scan_token(&mut self) -> Option<TokenType> {
        let ch = self.advance()?;

        let token: Option<TokenType> = match ch {
            // Single character
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
                if self.match_next_char('/') {
                    // Read ahead to end of line
                    while self.peek() != Some('\n') && !self.is_at_end() {
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
                if self.match_next_char('=') {
                    Some(TokenType::BangEqual)
                } else {
                    Some(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_next_char('=') {
                    Some(TokenType::EqualEqual)
                } else {
                    Some(TokenType::Equal)
                }
            }
            '>' => {
                if self.match_next_char('=') {
                    Some(TokenType::GreaterEqual)
                } else {
                    Some(TokenType::Greater)
                }
            }
            '<' => {
                if self.match_next_char('=') {
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
            // "" => Token::new(TokenType::Identifier, "", self.line),
            '"' => {
                let result = self.string();
                match result {
                    Ok(_) => None,
                    Err(err) => {
                        // This is weird. Fix this.
                        eprintln!("{}", err);
                        None
                    }
                }
            }
            '0'..='9' => {
                let result = self.number();
                match result {
                    Ok(_) => None,
                    Err(err) => {
                        // This is weird. Fix this.
                        eprintln!("[line {}] {}", self.line, err);
                        None
                    }
                }
            }

            // Keywords
            // "and" => Token::new(TokenType::And, "and", self.line),
            // "class" => Token::new(TokenType::Class, "class", self.line),
            // "else" => Token::new(TokenType::Else, "else", self.line),
            // "false" => Token::new(TokenType::False, "false", self.line),
            // "fun" => Token::new(TokenType::Fun, "fun", self.line),
            // "for" => Token::new(TokenType::For, "for", self.line),
            // "if" => Token::new(TokenType::If, "if", self.line),
            // "nil" => Token::new(TokenType::Nil, "nil", self.line),
            // "or" => Token::new(TokenType::Or, "or", self.line),
            // "print" => Token::new(TokenType::Print, "print", self.line),
            // "return" => Token::new(TokenType::Return, "return", self.line),
            // "super" => Token::new(TokenType::Super, "super", self.line),
            // "this" => Token::new(TokenType::This, "this", self.line),
            // "true" => Token::new(TokenType::True, "true", self.line),
            // "var" => Token::new(TokenType::Var, "var", self.line),
            // "while" => Token::new(TokenType::While, "while", self.line),
            // "eof" => Token::new(TokenType::Eof, "eof", self.line),
            _x => {
                println!("Implement me. :::: {}", _x);
                None
            }
        };

        token
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).cloned()
    }

    fn advance(&mut self) -> Option<char> {
        let peek = self.peek();
        if peek.is_some() {
            self.current += 1;
        }
        peek
    }

    fn match_next_char(&mut self, ch: char) -> bool {
        if self.peek() != Some(ch) {
            return false;
        }
        self.advance();
        true
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err("Unterminated String".into());
        }

        self.advance(); // consume ending quote

        // Trim surrounding quotes of string literal.
        self.tokens.push(Token::new(
            TokenType::String,
            &self.source[(self.start + 1)..(self.current - 1)],
            self.line,
        ));

        Ok(())
    }

    fn number(&mut self) -> Result<(), String> {
        let mut decimal = false;
        let mut res = Ok(());

        while self.peek_is_digit().is_some() && !self.is_at_end() {
            match self.peek().unwrap() {
                '0'..='9' => {}
                '.' => {
                    // we can only have 1 decimal point in a number
                    if decimal == true {
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

        if res.is_err() {
            return res;
        }

        self.tokens.push(Token::new(
            TokenType::Number,
            &self.source[self.start..self.current],
            self.line,
        ));

        Ok(())
    }

    fn peek_is_digit(&self) -> Option<bool> {
        let ch = self.peek()?;
        match ch {
            '0'..='9' => Some(true),
            '.' => Some(true),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_next_test() {
        let mut scanner = Scanner::new("// this is a comment");
        scanner.scan_tokens().unwrap();
    }
}
