use crate::errors;
use crate::tokens::{self, TokenType};

pub struct Scanner<'a> {
    source: &'a str,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner {
        Scanner { source }
    }

    // pub fn read_file(filename: &str) -> Scanner {

    //     Scanner {
    //         source
    //     }
    // }

    pub fn scan_tokens(&self) -> Result<Vec<&'a str>, errors::RloxError> {
        let splits = self.source.split(' ').collect::<Vec<&str>>();
        for split in splits {
            tokens::Token::new(split);
        }
        Ok(self.source.split(' ').collect::<Vec<&str>>())
    }
}
