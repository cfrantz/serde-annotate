// Document Enum for serialization
use std::convert::TryFrom;

use crate::error::Error;
use crate::integer::Int;
use crate::relax::Relax;

/// Represents possible serialized string formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StrFormat {
    /// The standard format for the serialization backend.
    Standard,
    /// Always quote the string, even if not required by the backend.
    Quoted,
    /// Render the string unquoted if allowed by the backend.
    Unquoted,
    /// Format the string as a multiline block, if allowed by the backend.
    Multiline,
}

/// Represents possible serialized bytes formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BytesFormat {
    /// The standard format for the serialization backend.
    Standard,
    /// Hexadecimal string (e.g. "98ab45cdeaff").
    HexStr,
    /// Hexdump like `hexdump -vC ...`.
    Hexdump,
    /// Hexdump like `xxd ...`.
    Xxd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommentFormat {
    /// The standard format for the serialization backend.
    Standard,
    /// Render comments in block form if allowed by the backend.
    Block,
    /// Render comments in single-line hash form if allowed by the backend.
    Hash,
    /// Render comments in single-line slash-slash form if allowed by the backend.
    SlashSlash,
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
    /// Parses a string into a `Document` using the maximally permissive parser.
    pub fn parse(text: &str) -> Result<Document, Error> {
        let relax = Relax::default();
        relax.from_str(text)
    }

    /// Parses a string into a `Document` using strict json.
    pub fn from_json(text: &str) -> Result<Document, Error> {
        let relax = Relax::json();
        relax.from_str(text)
    }

    /// Parses a string into a `Document` using json5.
    pub fn from_json5(text: &str) -> Result<Document, Error> {
        let relax = Relax::json5();
        relax.from_str(text)
    }

    /// Parses a string into a `Document` using hjson.
    pub fn from_hjson(text: &str) -> Result<Document, Error> {
        let relax = Relax::hjson();
        relax.from_str(text)
    }

    /// Returns the variant of this `Document`.
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

    /// Returns the list of fragments in this node.
    pub fn fragments(&self) -> Result<&[Document], Error> {
        if let Document::Fragment(f) = self {
            Ok(f)
        } else {
            Err(Error::StructureError("Fragment", self.variant()))
        }
    }

    /// Returns a mutable list of fragments in this node.
    pub fn fragments_mut(&mut self) -> Result<&mut [Document], Error> {
        if let Document::Fragment(ref mut f) = self {
            Ok(f)
        } else {
            Err(Error::StructureError("Fragment", self.variant()))
        }
    }

    /// Returns this node as a kvpair.
    pub fn as_kv(&self) -> Result<(&Document, &Document), Error> {
        let frags = self.fragments()?;
        let kv = frags.iter().filter(|f| f.has_value()).collect::<Vec<_>>();
        match kv.len() {
            0 => Err(Error::StructureError("kvpair", "zero elements")),
            1 => Err(Error::StructureError("kvpair", "one element")),
            2 => Ok((kv[0], kv[1])),
            _ => Err(Error::StructureError("kvpair", "many elements")),
        }
    }

    /// Returns this node as a mutable kvpair.
    pub fn as_kv_mut(&mut self) -> Result<(&mut Document, &mut Document), Error> {
        let frags = self.fragments_mut()?;
        let mut kv = frags
            .iter_mut()
            .filter(|f| f.has_value())
            .collect::<Vec<_>>();
        match kv.len() {
            0 => Err(Error::StructureError("kvpair", "zero elements")),
            1 => Err(Error::StructureError("kvpair", "one element")),
            2 => {
                let v = kv.pop().unwrap();
                let k = kv.pop().unwrap();
                Ok((k, v))
            }
            _ => Err(Error::StructureError("kvpair", "many elements")),
        }
    }

    /// Returns a reference to this node's value-containing `Document`.
    /// A comment node has no value and thus returns an error.
    /// A fragment node must contain exactly one value or it returns an error.
    pub fn as_value(&self) -> Result<&Document, Error> {
        match self {
            Document::Comment(_, _) => Err(Error::StructureError("a value", "Comment")),
            Document::Compact(c) => c.as_value(),
            Document::Fragment(frags) => {
                let values = frags.iter().filter(|f| f.has_value()).collect::<Vec<_>>();
                match values.len() {
                    0 => Err(Error::StructureError("one value", "zero")),
                    1 => Ok(values[0]),
                    _ => Err(Error::StructureError("one value", "many")),
                }
            }
            _ => Ok(self),
        }
    }

    /// Returns a mutable reference to this node's value-containing `Document`.
    /// A comment node has no value and thus returns an error.
    /// A fragment node must contain exactly one value or it returns an error.
    pub fn as_value_mut(&mut self) -> Result<&mut Document, Error> {
        match self {
            Document::Comment(_, _) => Err(Error::StructureError("a value", "Comment")),
            Document::Compact(c) => c.as_value_mut(),
            Document::Fragment(frags) => {
                let mut values = frags
                    .iter_mut()
                    .filter(|f| f.has_value())
                    .collect::<Vec<_>>();
                match values.len() {
                    0 => Err(Error::StructureError("one value", "zero")),
                    1 => Ok(values.pop().unwrap()),
                    _ => Err(Error::StructureError("one value", "many")),
                }
            }
            _ => Ok(self),
        }
    }

    /// Returns whether this node is a value-containing node.
    pub fn has_value(&self) -> bool {
        match self {
            Document::Comment(_, _) => false,
            Document::Compact(c) => c.has_value(),
            Document::Fragment(f) => f.iter().any(Document::has_value),
            _ => true,
        }
    }

    /// Returns the index of the last value containing node in a slice.
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

    /// Returns the comment information contained in a node.
    pub fn comment(&self) -> Option<(&str, &CommentFormat)> {
        if let Document::Comment(c, f) = self {
            Some((c.as_str(), f))
        } else {
            None
        }
    }

    /// Converts the document into a str slice or returns an error.
    pub fn as_str(&self) -> Result<&str, Error> {
        match self.as_value()? {
            Document::String(s, _) => Ok(s.as_str()),
            Document::StaticStr(s, _) => Ok(s),
            _ => Err(Error::StructureError("String", self.variant())),
        }
    }

    /// Converts the document into a null value or returns an error.
    pub fn as_null(&self) -> Result<(), Error> {
        match self.as_value()? {
            Document::Null => Ok(()),
            _ => Err(Error::StructureError("Null", self.variant())),
        }
    }
}

fn parse_bool(v: &str) -> Result<bool, Error> {
    match v {
        "true" | "True" | "TRUE" => Ok(true),
        "false" | "False" | "FALSE" => Ok(false),
        _ => Err(Error::StructureError("Boolean", "String")),
    }
}

/// Tries to convert the document into a boolean value.
impl TryFrom<&Document> for bool {
    type Error = Error;
    fn try_from(v: &Document) -> Result<Self, Self::Error> {
        match v.as_value()? {
            Document::Boolean(b) => Ok(*b),
            Document::String(s, _) => parse_bool(s.as_str()),
            Document::StaticStr(s, _) => parse_bool(s),
            _ => Err(Error::StructureError("Boolean", v.variant())),
        }
    }
}

/// Tries to convert the document into a char value.
impl TryFrom<&Document> for char {
    type Error = Error;
    fn try_from(v: &Document) -> Result<Self, Self::Error> {
        let s = v.as_str()?;
        let mut chars = s.chars();
        let ch = chars
            .next()
            .ok_or(Error::StructureError("one character", "zero"))?;
        if chars.next().is_some() {
            return Err(Error::StructureError("one character", "many"));
        }
        Ok(ch)
    }
}

macro_rules! impl_int_conv {
    ($t:ty) => {
        /// Tries to convert the document into an integer value.
        impl TryFrom<&Document> for $t {
            type Error = Error;
            fn try_from(v: &Document) -> Result<Self, Self::Error> {
                match v.as_value()? {
                    Document::Int(v) => Ok(<$t>::from(v)),
                    Document::Float(v) => Ok(*v as $t),
                    Document::String(s, _) => Ok(<$t>::from(Int::from_str_radix(s.as_str(), 0)?)),
                    Document::StaticStr(s, _) => Ok(<$t>::from(Int::from_str_radix(s, 0)?)),
                    _ => Err(Error::StructureError("Int", v.variant())),
                }
            }
        }
    };
}
impl_int_conv!(u8);
impl_int_conv!(u16);
impl_int_conv!(u32);
impl_int_conv!(u64);
impl_int_conv!(u128);
impl_int_conv!(i8);
impl_int_conv!(i16);
impl_int_conv!(i32);
impl_int_conv!(i64);
impl_int_conv!(i128);

macro_rules! impl_float_conv {
    ($t:ty) => {
        /// Tries to convert the document into a float value.
        impl TryFrom<&Document> for $t {
            type Error = Error;
            fn try_from(v: &Document) -> Result<Self, Self::Error> {
                match v.as_value()? {
                    Document::Int(v) => Ok(<$t>::from(v)),
                    Document::Float(v) => Ok(*v as $t),
                    _ => Err(Error::StructureError("Float", v.variant())),
                }
            }
        }
    };
}
impl_float_conv!(f32);
impl_float_conv!(f64);
