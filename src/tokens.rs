use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug)]
enum TokenLiteral {
    Str(String),
    Number(f64),
    None,
}

#[derive(Debug)]
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
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.token_type, self.lexeme)
    }
}
