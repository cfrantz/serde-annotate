// Document Enum for serialization
//
//
use crate::integer::Int;

/// Represents possible serialized string formats.
#[derive(Clone, Copy, Debug)]
pub enum StrFormat {
    /// The standard format for the serialization backend.
    Standard,
    /// Always quote the string, even if not required by the backend.
    Quoted,
    /// Format the string as a multiline block, if allowed by the backend.
    Multiline,
}

#[derive(Clone, Debug)]
pub struct KeyValue(pub Document, pub Document);

#[derive(Clone, Debug)]
pub enum Document {
    Comment(String),
    String(String, StrFormat),
    Boolean(bool),
    Int(Int),
    Float(f64),
    Mapping(Vec<KeyValue>),
    Sequence(Vec<Document>),
    Bytes(Vec<u8>),
    Null,
}
