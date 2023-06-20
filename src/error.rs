use crate::relax::ParseError;
use serde::{de, ser};
use std::char::CharTryFromError;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::ParseBoolError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serializer error: {0}")]
    Serialize(String),
    #[error("deserializer error: {0}")]
    Deserialize(String),
    #[error("unknown error: {0}")]
    Unknown(String),
    #[error("unhandled escape: `\\{0}`")]
    EscapeError(char),
    #[error("formatter error: {0:?}")]
    FmtError(std::fmt::Error),
    #[error("Hexdump error: {0}")]
    HexdumpError(String),
    #[error("Type {0:?} is not valid as a mapping key")]
    KeyTypeError(&'static str),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    ParseBoolError(#[from] ParseBoolError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    CharTryFromError(#[from] CharTryFromError),
    #[error("document structure error: expected {0} but got {1}")]
    StructureError(&'static str, &'static str),
    #[error("syntax error: {0} at {1}:{col}\n| {3}\n| {4:>col$}", col = .2)]
    SyntaxError(String, usize, usize, String, &'static str),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serialize(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Deserialize(msg.to_string())
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        Error::FmtError(e)
    }
}
