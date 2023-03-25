use std::error::Error;
use std::fmt::{Display, Formatter, write};

#[derive(Debug)]
pub enum H2RustError {
    SQLError(String),

}

impl Display for H2RustError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            H2RustError::SQLError(s) => write!(f, "{}", s),
            _ => write!(f, "{}", "未分类的error")
        }
    }
}

impl Error for H2RustError {}

