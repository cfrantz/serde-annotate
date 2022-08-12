use crate::relax::ParseError;
use serde::ser;
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serializer error: {0}")]
    Ser(String),
    #[error("unknown error: {0}")]
    Unknown(String),
    #[error("formatter error: {0:?}")]
    FmtError(std::fmt::Error),
    #[error("Type {0:?} is not valid as a mapping key")]
    KeyTypeError(&'static str),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("document structure error: expected {0} but got {1}")]
    StructureError(&'static str, &'static str),
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Ser(msg.to_string())
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        Error::FmtError(e)
    }
}
