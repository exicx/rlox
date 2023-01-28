use std::error;
use std::fmt;

#[derive(Debug)]
pub enum RloxError {
    Cmdline(String),
    Scan {
        line: usize,
        help: String,
        message: String,
    },
    Parse(String),
}

impl fmt::Display for RloxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
impl error::Error for RloxError {}
