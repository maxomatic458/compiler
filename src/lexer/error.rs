use crate::error::CompilerError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TokenError {
    #[error("unknown token: {0}")]
    UnknownToken(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum LexerError {
    #[error("{0}")]
    InvalidSyntax(String),
    #[error("{0}")]
    IllegalIdentifier(String),
}

impl CompilerError for LexerError {
    fn id(&self) -> u32 {
        match self {
            LexerError::InvalidSyntax(..) => 0,
            LexerError::IllegalIdentifier(..) => 1,
        }
    }

    fn name(&self) -> &str {
        match self {
            LexerError::InvalidSyntax(..) => "invalid syntax",
            LexerError::IllegalIdentifier(..) => "illegal identifier",
        }
    }

    fn err_msg(&self) -> String {
        self.to_string()
    }
}
