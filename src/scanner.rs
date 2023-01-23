use crate::errors;
use crate::tokens::{Token, TokenType};

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token<'a>>,
    line: i32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner {
        Scanner {
            source,
            tokens: vec![],
            line: 0,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), errors::RloxError> {
        let splits = self.source.split(' ').collect::<Vec<&str>>();

        for split in splits {
            self.scan_token(&split);
        }

        self.tokens.push(Token::new(TokenType::Eof, "", self.line));
        Ok(())
    }

    fn scan_token(&mut self, lex: &str) -> Result<(), errors::RloxError> {
        let token = match lex {
            "and" => Token::new(TokenType::And, "and", 0),
            _ => {
                return Err(errors::RloxError::ScanError {
                    line: self.line,
                    help: "split".into(),
                    message: format!("Unable to scan token: {}", lex),
                })
            }
        };

        self.tokens.push(token);

        Ok(())
    }

    // Thinking ....
    fn add_token(&mut self, token: TokenType, lex: &'static str) {
        let token = Token::new(token, lex, self.line);
        self.tokens.push(token);
    }
}
