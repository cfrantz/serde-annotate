use serde::ser;

use crate::document::{Document, KeyValue, StrFormat};
use crate::error::Error;
use crate::integer::{Base, Int};

/// Serializer adapter that adds user-annotatons to the serialized document.
pub struct AnnotatedSerializer {
    base: Base,
    strformat: StrFormat,
}

impl AnnotatedSerializer {
    pub fn new() -> Self {
        AnnotatedSerializer {
            base: Base::Dec,
            strformat: StrFormat::Standard,
        }
    }
}

impl<'a> ser::Serializer for &'a mut AnnotatedSerializer {
    type Ok = Document;
    type Error = Error;

    type SerializeSeq = SerializeSeq<'a>;
    type SerializeTuple = SerializeTuple<'a>;
    type SerializeTupleStruct = SerializeTupleStruct<'a>;
    type SerializeTupleVariant = SerializeTupleVariant<'a>;
    type SerializeMap = SerializeMap<'a>;
    type SerializeStruct = SerializeStruct<'a>;
    type SerializeStructVariant = SerializeStructVariant<'a>;

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
        Ok(Document::Float(v as f64))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Document::String(v.to_string(), self.strformat))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Document::String(v.to_string(), self.strformat))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Null)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
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
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
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
        let kv = KeyValue(
            Document::String(variant.to_string(), StrFormat::Standard),
            value.serialize(self)?,
        );
        Ok(Document::Mapping(vec![kv]))
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

pub struct SerializeSeq<'a> {
    serializer: &'a mut AnnotatedSerializer,
    sequence: Vec<Document>,
}

impl<'a> SerializeSeq<'a> {
    fn new(s: &'a mut AnnotatedSerializer) -> Self {
        SerializeSeq {
            serializer: s,
            sequence: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.sequence.push(value.serialize(&mut *self.serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Sequence(self.sequence))
    }
}

pub struct SerializeTuple<'a> {
    serializer: &'a mut AnnotatedSerializer,
    sequence: Vec<Document>,
}

impl<'a> SerializeTuple<'a> {
    fn new(s: &'a mut AnnotatedSerializer) -> Self {
        SerializeTuple {
            serializer: s,
            sequence: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeTuple for SerializeTuple<'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.sequence.push(value.serialize(&mut *self.serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Sequence(self.sequence))
    }
}

pub struct SerializeTupleStruct<'a> {
    serializer: &'a mut AnnotatedSerializer,
    sequence: Vec<Document>,
}

impl<'a> SerializeTupleStruct<'a> {
    fn new(s: &'a mut AnnotatedSerializer) -> Self {
        SerializeTupleStruct {
            serializer: s,
            sequence: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeTupleStruct for SerializeTupleStruct<'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.sequence.push(value.serialize(&mut *self.serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Sequence(self.sequence))
    }
}

pub struct SerializeTupleVariant<'a> {
    serializer: &'a mut AnnotatedSerializer,
    variant: &'static str,
    sequence: Vec<Document>,
}

impl<'a> SerializeTupleVariant<'a> {
    fn new(s: &'a mut AnnotatedSerializer, v: &'static str) -> Self {
        SerializeTupleVariant {
            serializer: s,
            variant: v,
            sequence: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeTupleVariant for SerializeTupleVariant<'a> {
    type Ok = Document;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.sequence.push(value.serialize(&mut *self.serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Mapping(vec![KeyValue(
            Document::String(self.variant.to_string(), StrFormat::Standard),
            Document::Sequence(self.sequence),
        )]))
    }
}

pub struct SerializeMap<'a> {
    serializer: &'a mut AnnotatedSerializer,
    next_key: Option<Document>,
    mapping: Vec<KeyValue>,
}

impl<'a> SerializeMap<'a> {
    fn new(s: &'a mut AnnotatedSerializer) -> Self {
        SerializeMap {
            serializer: s,
            next_key: None,
            mapping: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeMap for SerializeMap<'a> {
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
            Some(key) => self
                .mapping
                .push(KeyValue(key, value.serialize(&mut *self.serializer)?)),
            None => panic!("serialize_value called before serialize_key"),
        };
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + ser::Serialize,
        V: ?Sized + ser::Serialize,
    {
        self.mapping.push(KeyValue(
            key.serialize(&mut *self.serializer)?,
            value.serialize(&mut *self.serializer)?,
        ));
        Ok(())
    }
}

pub struct SerializeStruct<'a> {
    serializer: &'a mut AnnotatedSerializer,
    mapping: Vec<KeyValue>,
}

impl<'a> SerializeStruct<'a> {
    fn new(s: &'a mut AnnotatedSerializer) -> Self {
        SerializeStruct {
            serializer: s,
            mapping: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeStruct for SerializeStruct<'a> {
    type Ok = Document;
    type Error = Error;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Mapping(self.mapping))
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.mapping.push(KeyValue(
            Document::String(key.to_string(), StrFormat::Standard),
            value.serialize(&mut *self.serializer)?,
        ));
        Ok(())
    }
}

pub struct SerializeStructVariant<'a> {
    serializer: &'a mut AnnotatedSerializer,
    variant: &'static str,
    mapping: Vec<KeyValue>,
}

impl<'a> SerializeStructVariant<'a> {
    fn new(s: &'a mut AnnotatedSerializer, v: &'static str) -> Self {
        SerializeStructVariant {
            serializer: s,
            variant: v,
            mapping: Vec::new(),
        }
    }
}

impl<'a> ser::SerializeStructVariant for SerializeStructVariant<'a> {
    type Ok = Document;
    type Error = Error;

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Document::Mapping(vec![KeyValue(
            Document::String(self.variant.to_string(), StrFormat::Standard),
            Document::Mapping(self.mapping),
        )]))
    }

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.mapping.push(KeyValue(
            Document::String(key.to_string(), StrFormat::Standard),
            value.serialize(&mut *self.serializer)?,
        ));
        Ok(())
    }
}
