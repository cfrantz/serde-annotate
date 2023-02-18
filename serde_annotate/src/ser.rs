use serde::ser;

use crate::annotate::{Annotate, Format, MemberId};
use crate::document::{BytesFormat, CommentFormat, Document, StrFormat};
use crate::error::Error;
use crate::hexdump;
use crate::integer::{Base, Int};

pub fn serialize<T>(value: &T) -> Result<Document, Error>
where
    T: ?Sized + ser::Serialize,
{
    let mut ser = AnnotatedSerializer::new(value.as_annotate());
    value.serialize(&mut ser)
}

/// Serializer adapter that adds user-annotatons to the serialized document.
#[derive(Clone)]
pub struct AnnotatedSerializer<'a> {
    annotator: Option<&'a dyn Annotate>,
    base: Base,
    strformat: StrFormat,
    bytesformat: BytesFormat,
    compact: bool,
}

impl<'a> AnnotatedSerializer<'a> {
    pub fn new(annotator: Option<&'a dyn Annotate>) -> Self {
        AnnotatedSerializer {
            annotator,
            base: Base::Dec,
            strformat: StrFormat::Standard,
            bytesformat: BytesFormat::Standard,
            compact: false,
        }
    }

    fn with_base(&self, b: Base) -> Self {
        let mut x = self.clone();
        x.base = b;
        x
    }

    fn with_bytesformat(&self, b: BytesFormat) -> Self {
        let mut x = self.clone();
        x.bytesformat = b;
        x
    }

    fn with_strformat(&self, s: StrFormat) -> Self {
        let mut x = self.clone();
        x.strformat = s;
        x
    }

    fn with_compact(&self, c: bool) -> Self {
        let mut x = self.clone();
        x.compact = c;
        x
    }

    fn annotate(&self, variant: Option<&str>, field: &MemberId) -> Option<Self> {
        match self.annotator.and_then(|a| a.format(variant, field)) {
            Some(Format::Block) => Some(self.with_strformat(StrFormat::Multiline)),
            Some(Format::Binary) => Some(self.with_base(Base::Bin)),
            Some(Format::Decimal) => Some(self.with_base(Base::Dec)),
            Some(Format::Hex) => Some(self.with_base(Base::Hex)),
            Some(Format::Octal) => Some(self.with_base(Base::Oct)),
            Some(Format::Compact) => Some(self.with_compact(true)),
            Some(Format::HexStr) => Some(self.with_bytesformat(BytesFormat::HexStr)),
            Some(Format::Hexdump) => Some(self.with_bytesformat(BytesFormat::Hexdump)),
            Some(Format::Xxd) => Some(self.with_bytesformat(BytesFormat::Xxd)),
            None => None,
        }
    }

    fn comment(&self, variant: Option<&str>, field: &MemberId) -> Option<Document> {
        self.annotator
            .and_then(|a| a.comment(variant, field))
            .map(|c| Document::Comment(c, CommentFormat::Standard))
    }

    fn serialize<T>(&self, value: &T, ser: Option<AnnotatedSerializer>) -> Result<Document, Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let mut ser = ser.unwrap_or(self.clone());
        ser.annotator = value.as_annotate();
        value.serialize(&mut ser)
    }
}

impl<'s, 'a> ser::Serializer for &'s mut AnnotatedSerializer<'a> {
    type Ok = Document;
    type Error = Error;

    type SerializeSeq = SerializeSeq<'s, 'a>;
    type SerializeTuple = SerializeTuple<'s, 'a>;
    type SerializeTupleStruct = SerializeTupleStruct<'s, 'a>;
    type SerializeTupleVariant = SerializeTupleVariant<'s, 'a>;
    type SerializeMap = SerializeMap<'s, 'a>;
    type SerializeStruct = SerializeStruct<'s, 'a>;
    type SerializeStructVariant = SerializeStructVariant<'s, 'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Boolean(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Int(Int::new(v, self.base)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Document::String(v.to_string(), self.strformat))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Document::String(v.to_string(), self.strformat))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if let Some(string) = hexdump::to_string(v, self.bytesformat) {
            Ok(Document::String(
                string,
                if self.bytesformat == BytesFormat::HexStr {
                    StrFormat::Standard
                } else {
                    StrFormat::Multiline
                },
            ))
        } else {
            Ok(Document::Bytes(v.to_vec()))
        }
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Null)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.serialize(value, None)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let node = self.serialize_str(variant)?;
        // TODO(serde-annotate#6): currently, placing a comment on a unit variant results in
        // ugly (json) or bad (yaml) documents.  For now, omit comments on
        // unit variants until we refactor comment emitting.
        //if let Some(c) = self.comment(Some(variant), &MemberId::Variant) {
        //    Ok(Document::Fragment(vec![c, node]))
        //} else {
        //    Ok(node)
        //}
        Ok(node)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let field = MemberId::Index(0);
        let node = self.serialize(value, self.annotate(None, &field))?;
        // TODO(serde-annotate#6): currently, placing a comment on a newtype structs results in
        // ugly (json) or bad (yaml) documents.  For now, omit comments on
        // unit variants until we refactor comment emitting.
        //if let Some(c) = self.comment(None, &field) {
        //    Ok(Document::Fragment(vec![c, node]))
        //} else {
        //    Ok(node)
        //}
        Ok(node)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let a = self.annotate(Some(variant), &MemberId::Variant);
        let compact = a.map(|a| a.compact).unwrap_or(false);
        let v = self.serialize(value, self.annotate(Some(variant), &MemberId::Index(0)))?;
        let v = if compact {
            Document::Compact(v.into())
        } else {
            v
        };
        let mut nodes = vec![];
        if let Some(c) = self.comment(Some(variant), &MemberId::Variant) {
            nodes.push(c);
        }
        nodes.push(Document::from(variant));
        nodes.push(v);

        Ok(Document::Mapping(vec![Document::Fragment(nodes)]))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeTuple::new(self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeTupleStruct::new(self))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant::new(self, variant))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap::new(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct::new(self))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant::new(self, variant))
    }
}

pub struct SerializeSeq<'s, 'a> {
    serializer: &'s mut AnnotatedSerializer<'a>,
    sequence: Vec<Document>,
}

impl<'s, 'a> SerializeSeq<'s, 'a> {
    fn new(s: &'s mut AnnotatedSerializer<'a>) -> Self {
        SerializeSeq {
            serializer: s,
            sequence: Vec::new(),
        }
    }
}

impl<'s, 'a> ser::SerializeSeq for SerializeSeq<'s, 'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.sequence.push(self.serializer.serialize(value, None)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Sequence(self.sequence))
    }
}

pub struct SerializeTuple<'s, 'a> {
    serializer: &'s mut AnnotatedSerializer<'a>,
    sequence: Vec<Document>,
}

impl<'s, 'a> SerializeTuple<'s, 'a> {
    fn new(s: &'s mut AnnotatedSerializer<'a>) -> Self {
        SerializeTuple {
            serializer: s,
            sequence: Vec::new(),
        }
    }
}

impl<'s, 'a> ser::SerializeTuple for SerializeTuple<'s, 'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.sequence.push(self.serializer.serialize(value, None)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Sequence(self.sequence))
    }
}

pub struct SerializeTupleStruct<'s, 'a> {
    serializer: &'s mut AnnotatedSerializer<'a>,
    index: u32,
    sequence: Vec<Document>,
}

impl<'s, 'a> SerializeTupleStruct<'s, 'a> {
    fn new(s: &'s mut AnnotatedSerializer<'a>) -> Self {
        SerializeTupleStruct {
            serializer: s,
            index: 0,
            sequence: Vec::new(),
        }
    }
}

impl<'s, 'a> ser::SerializeTupleStruct for SerializeTupleStruct<'s, 'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let field = MemberId::Index(self.index);
        let node = self
            .serializer
            .serialize(value, self.serializer.annotate(None, &field))?;
        if let Some(c) = self.serializer.comment(None, &field) {
            self.sequence.push(Document::Fragment(vec![c, node]));
        } else {
            self.sequence.push(node);
        }
        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Sequence(self.sequence))
    }
}

pub struct SerializeTupleVariant<'s, 'a> {
    serializer: &'s mut AnnotatedSerializer<'a>,
    variant: &'static str,
    index: u32,
    sequence: Vec<Document>,
}

impl<'s, 'a> SerializeTupleVariant<'s, 'a> {
    fn new(s: &'s mut AnnotatedSerializer<'a>, v: &'static str) -> Self {
        SerializeTupleVariant {
            serializer: s,
            variant: v,
            index: 0,
            sequence: Vec::new(),
        }
    }
}

impl<'s, 'a> ser::SerializeTupleVariant for SerializeTupleVariant<'s, 'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let field = MemberId::Index(self.index);
        let node = self
            .serializer
            .serialize(value, self.serializer.annotate(Some(self.variant), &field))?;
        if let Some(c) = self.serializer.comment(Some(self.variant), &field) {
            self.sequence.push(Document::Fragment(vec![c, node]));
        } else {
            self.sequence.push(node);
        }

        self.index += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let a = self
            .serializer
            .annotate(Some(self.variant), &MemberId::Variant);
        let compact = a.map(|a| a.compact).unwrap_or(false);
        let sequence = if compact {
            Document::Compact(Document::Sequence(self.sequence).into())
        } else {
            Document::Sequence(self.sequence)
        };
        let mut nodes = vec![];
        if let Some(c) = self
            .serializer
            .comment(Some(self.variant), &MemberId::Variant)
        {
            nodes.push(c);
        }
        nodes.push(Document::from(self.variant));
        nodes.push(sequence);
        Ok(Document::Mapping(vec![Document::Fragment(nodes)]))
    }
}

pub struct SerializeMap<'s, 'a> {
    serializer: &'s mut AnnotatedSerializer<'a>,
    next_key: Option<Document>,
    mapping: Vec<Document>,
}

impl<'s, 'a> SerializeMap<'s, 'a> {
    fn new(s: &'s mut AnnotatedSerializer<'a>) -> Self {
        SerializeMap {
            serializer: s,
            next_key: None,
            mapping: Vec::new(),
        }
    }
}

impl<'s, 'a> ser::SerializeMap for SerializeMap<'s, 'a> {
    type Ok = Document;
    type Error = Error;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Mapping(self.mapping))
    }

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.next_key = Some(key.serialize(&mut *self.serializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        match self.next_key.take() {
            Some(key) => {
                self.mapping.push(Document::Fragment(vec![
                    key,
                    self.serializer.serialize(value, None)?,
                ]));
            }
            None => panic!("serialize_value called before serialize_key"),
        };
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + ser::Serialize,
        V: ?Sized + ser::Serialize,
    {
        self.mapping.push(Document::Fragment(vec![
            key.serialize(&mut *self.serializer)?,
            self.serializer.serialize(value, None)?,
        ]));
        Ok(())
    }
}

pub struct SerializeStruct<'s, 'a> {
    serializer: &'s mut AnnotatedSerializer<'a>,
    mapping: Vec<Document>,
}

impl<'s, 'a> SerializeStruct<'s, 'a> {
    fn new(s: &'s mut AnnotatedSerializer<'a>) -> Self {
        SerializeStruct {
            serializer: s,
            mapping: Vec::new(),
        }
    }
}

impl<'s, 'a> ser::SerializeStruct for SerializeStruct<'s, 'a> {
    type Ok = Document;
    type Error = Error;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Mapping(self.mapping))
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let field = MemberId::Name(key);
        let mut nodes = vec![];
        if let Some(c) = self.serializer.comment(None, &field) {
            nodes.push(c);
        }
        nodes.push(Document::from(key));
        nodes.push(
            self.serializer
                .serialize(value, self.serializer.annotate(None, &field))?,
        );
        self.mapping.push(Document::Fragment(nodes));
        Ok(())
    }
}

pub struct SerializeStructVariant<'s, 'a> {
    serializer: &'s mut AnnotatedSerializer<'a>,
    variant: &'static str,
    mapping: Vec<Document>,
}

impl<'s, 'a> SerializeStructVariant<'s, 'a> {
    fn new(s: &'s mut AnnotatedSerializer<'a>, v: &'static str) -> Self {
        SerializeStructVariant {
            serializer: s,
            variant: v,
            mapping: Vec::new(),
        }
    }
}

impl<'s, 'a> ser::SerializeStructVariant for SerializeStructVariant<'s, 'a> {
    type Ok = Document;
    type Error = Error;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let a = self
            .serializer
            .annotate(Some(self.variant), &MemberId::Variant);
        let compact = a.map(|a| a.compact).unwrap_or(false);
        let mapping = if compact {
            Document::Compact(Document::Mapping(self.mapping).into())
        } else {
            Document::Mapping(self.mapping)
        };
        let mut nodes = vec![];
        if let Some(c) = self
            .serializer
            .comment(Some(self.variant), &MemberId::Variant)
        {
            nodes.push(c);
        }
        nodes.push(Document::from(self.variant));
        nodes.push(mapping);
        Ok(Document::Mapping(vec![Document::Fragment(nodes)]))
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let field = MemberId::Name(key);
        let mut nodes = vec![];
        if let Some(c) = self.serializer.comment(None, &field) {
            nodes.push(c);
        }
        nodes.push(Document::from(key));
        nodes.push(
            self.serializer
                .serialize(value, self.serializer.annotate(None, &field))?,
        );
        self.mapping.push(Document::Fragment(nodes));
        Ok(())
    }
}
