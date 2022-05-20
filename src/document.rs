// Document Enum for serialization
//
//
use crate::integer::Int;

/// Represents possible serialized string formats.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    // A comment (emitted for humans, ignored by parsers).
    Comment(String),
    // A string value and its preferred formatting.
    String(String, StrFormat),
    // A boolean value.
    Boolean(bool),
    // An Integer (signed, unsigned, 8 to 128 bits) and its preferred output form.
    Int(Int),
    // Floating point types.
    Float(f64),
    // A mapping object (e.g. dict/hash/etc)
    Mapping(Vec<KeyValue>),
    // A sequence objecct (e.g. list/array/etc)
    Sequence(Vec<Document>),
    // A special form for bytes objects.
    Bytes(Vec<u8>),
    // A null value.
    Null,
    // A hint to the emitter to emit in compact form.
    Compact(Box<Document>),
}
