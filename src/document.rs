// Document Enum for serialization
//
//
use crate::error::Error;
use crate::integer::Int;
use crate::relax::Relax;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CommentFormat {
    Normal,
}

#[derive(Clone, Debug)]
pub enum Document {
    // A comment (emitted for humans, ignored by parsers).
    Comment(String, CommentFormat),
    // A string value and its preferred formatting.
    String(String, StrFormat),
    // A string reference and its preferred formatting.
    StaticStr(&'static str, StrFormat),
    // A boolean value.
    Boolean(bool),
    // An Integer (signed, unsigned, 8 to 128 bits) and its preferred output form.
    Int(Int),
    // Floating point types.
    Float(f64),
    // A mapping object (e.g. dict/hash/etc)
    Mapping(Vec<Document>),
    // A sequence objecct (e.g. list/array/etc)
    Sequence(Vec<Document>),
    // A special form for bytes objects.
    Bytes(Vec<u8>),
    // A null value.
    Null,
    // A hint to the emitter to emit in compact form.
    Compact(Box<Document>),
    // A fragment holds a set of document nodes that may be useful as an
    // aggregate, such as Key-Value pairs.
    Fragment(Vec<Document>),
}

impl From<&'static str> for Document {
    fn from(s: &'static str) -> Self {
        Document::StaticStr(s, StrFormat::Standard)
    }
}

impl Document {
    pub fn parse(text: &str) -> Result<Document, Error> {
        let relax = Relax::default();
        relax.from_str(text)
    }

    pub fn variant(&self) -> &'static str {
        match self {
            Document::Comment(_, _) => "Comment",
            Document::String(_, _) => "String",
            Document::StaticStr(_, _) => "StaticStr",
            Document::Boolean(_) => "Boolean",
            Document::Int(_) => "Int",
            Document::Float(_) => "Float",
            Document::Mapping(_) => "Mapping",
            Document::Sequence(_) => "Sequence",
            Document::Bytes(_) => "Bytes",
            Document::Null => "Null",
            Document::Compact(_) => "Compact",
            Document::Fragment(_) => "Fragment",
        }
    }

    pub fn fragments(&self) -> Result<&[Document], Error> {
        if let Document::Fragment(f) = self {
            Ok(f)
        } else {
            Err(Error::StructureError("Fragment", self.variant()))
        }
    }

    pub fn as_kv(&self) -> Result<(&Document, &Document), Error> {
        let f = self.fragments()?;
        let kv = f
            .iter()
            .filter(|f| f.comment().is_none())
            .collect::<Vec<_>>();
        if kv.len() == 2 {
            Ok((kv[0], kv[1]))
        } else {
            Err(Error::StructureError("2 elements", "??"))
        }
    }

    pub fn has_value(&self) -> bool {
        match self {
            Document::Comment(_, _) => false,
            Document::Compact(c) => c.has_value(),
            Document::Fragment(f) => f.iter().any(Document::has_value),
            _ => true,
        }
    }

    pub fn last_value_index(sequence: &[Document]) -> usize {
        let mut last = sequence.len();
        for (i, frag) in sequence.iter().enumerate().rev() {
            if frag.has_value() {
                last = i;
                break;
            }
        }
        last
    }

    pub fn comment(&self) -> Option<(&str, &CommentFormat)> {
        if let Document::Comment(c, f) = self {
            Some((c.as_str(), f))
        } else {
            None
        }
    }
}
