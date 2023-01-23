use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum RloxError {
    CmdlineError(String),
    ScanError {
        line: i32,
        help: String,
        message: String,
    },
}
impl Display for RloxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
impl Error for RloxError {}
