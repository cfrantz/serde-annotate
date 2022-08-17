// Deserializer for serde-annotate `Document`s.

use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};

use crate::document::Document;
use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

/// A `Deserializer` deserializes a parsed document.
pub struct Deserializer<'de> {
    doc: &'de Document,
}

impl<'de> Deserializer<'de> {
    /// Creates a `Deserializer` from a parsed document.
    pub fn from_document(doc: &'de Document) -> Result<Self> {
        Ok(Deserializer {
            doc: doc.as_value()?,
        })
    }
}

/// Parses and deserializes a `str` into a `T`.  The parser is
/// maximally permissive.
pub fn from_str<T>(text: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let doc = Document::parse(text)?;
    let mut ds = Deserializer::from_document(&doc)?;
    T::deserialize(&mut ds)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }
    fn deserialize_ignored_any<V>(self, _v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_bool<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_bool(self.doc.try_into()?)
    }
    fn deserialize_u8<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_u8(self.doc.try_into()?)
    }
    fn deserialize_u16<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_u16(self.doc.try_into()?)
    }
    fn deserialize_u32<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_u32(self.doc.try_into()?)
    }
    fn deserialize_u64<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_u64(self.doc.try_into()?)
    }
    fn deserialize_u128<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_u128(self.doc.try_into()?)
    }

    fn deserialize_i8<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_i8(self.doc.try_into()?)
    }
    fn deserialize_i16<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_i16(self.doc.try_into()?)
    }
    fn deserialize_i32<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_i32(self.doc.try_into()?)
    }
    fn deserialize_i64<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_i64(self.doc.try_into()?)
    }
    fn deserialize_i128<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_i128(self.doc.try_into()?)
    }
    fn deserialize_f32<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_f32(self.doc.try_into()?)
    }
    fn deserialize_f64<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_f32(self.doc.try_into()?)
    }
    fn deserialize_char<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_char(self.doc.try_into()?)
    }
    fn deserialize_str<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_borrowed_str(self.doc.as_str()?)
    }
    fn deserialize_string<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(v)
    }

    fn deserialize_bytes<V>(self, _v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }
    fn deserialize_byte_buf<V>(self, _v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    fn deserialize_option<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.doc.as_value()? {
            Document::Null => v.visit_none(),
            _ => v.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.doc.as_null()?;
        v.visit_unit()
    }
    fn deserialize_unit_struct<V>(self, _name: &'static str, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(v)
    }
    fn deserialize_newtype_struct<V>(self, _name: &'static str, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        v.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Document::Sequence(seq) = self.doc {
            v.visit_seq(Sequence::new(seq.iter().filter(|f| f.has_value())))
        } else {
            Err(Error::StructureError("Sequence", self.doc.variant()))
        }
    }
    fn deserialize_tuple<V>(self, _len: usize, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(v)
    }
    fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(v)
    }

    fn deserialize_map<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Document::Mapping(map) = self.doc {
            v.visit_map(Sequence::new(map.iter().filter(|f| f.has_value())))
        } else {
            Err(Error::StructureError("Mapping", self.doc.variant()))
        }
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        v: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(v)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        v: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.doc.as_value()? {
            Document::String(s, _) => v.visit_enum(s.as_str().into_deserializer()),
            Document::StaticStr(s, _) => v.visit_enum(s.into_deserializer()),
            Document::Mapping(frags) => v.visit_enum(Enum::new(frags)?),
            _ => Err(Error::StructureError(
                "String or Mapping",
                self.doc.variant(),
            )),
        }
    }
    fn deserialize_identifier<V>(self, v: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(v)
    }
}

// The `Sequence` struct is used to provide sequence and map access to
// `Document::Sequence` and `Document::Mapping` nodes.
struct Sequence<'de, T: Iterator<Item = &'de Document>> {
    iter: T,
    value: Option<&'de Document>,
}

impl<'de, T: Iterator<Item = &'de Document>> Sequence<'de, T> {
    fn new<I: IntoIterator<Item = T::Item, IntoIter = T>>(ii: I) -> Self {
        Sequence {
            iter: ii.into_iter(),
            value: None,
        }
    }
}

impl<'de, T: Iterator<Item = &'de Document>> SeqAccess<'de> for Sequence<'de, T> {
    type Error = Error;

    fn next_element_seed<E>(&mut self, seed: E) -> Result<Option<E::Value>>
    where
        E: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(next) => seed
                .deserialize(&mut Deserializer::from_document(next)?)
                .map(Some),
            None => Ok(None),
        }
    }
}

impl<'de, T: Iterator<Item = &'de Document>> MapAccess<'de> for Sequence<'de, T> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(doc) => {
                let (k, v) = doc.as_kv()?;
                self.value = Some(v);
                seed.deserialize(&mut Deserializer::from_document(k)?)
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(v) => seed.deserialize(&mut Deserializer::from_document(v)?),
            None => Err(Error::Unknown("kvpair missing the value".into())),
        }
    }
}

// The `Enum` struct is used to provide access to the different enum kinds
// supported by the serde data model.
struct Enum<'de> {
    enm: &'de Document,
    var: &'de Document,
}

impl<'de> Enum<'de> {
    fn new(ev: &'de [Document]) -> Result<Self> {
        let (e, v) = match ev.len() {
            0 => Err(Error::StructureError("one value", "none")),
            1 => ev[0].as_kv(),
            _ => Err(Error::StructureError("one value", "many")),
        }?;
        Ok(Enum { enm: e, var: v })
    }
}

impl<'de> EnumAccess<'de> for Enum<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        Ok((
            seed.deserialize(&mut Deserializer::from_document(self.enm)?)?,
            self,
        ))
    }
}

impl<'de> VariantAccess<'de> for Enum<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::Unknown("unreachable: unit_variant".into()))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_document(self.var)?)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut Deserializer::from_document(self.var)?, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(&mut Deserializer::from_document(self.var)?, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
        }

        let j = r#"{"int":1,"seq":["a","b"]}"#;
        let expected = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
        };
        assert_eq!(expected, from_str(j).unwrap());
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let j = r#""Unit""#;
        let expected = E::Unit;
        assert_eq!(expected, from_str(j).unwrap());

        let j = r#"{"Newtype":1}"#;
        let expected = E::Newtype(1);
        assert_eq!(expected, from_str(j).unwrap());

        let j = r#"{"Tuple":[1,2]}"#;
        let expected = E::Tuple(1, 2);
        assert_eq!(expected, from_str(j).unwrap());

        let j = r#"{"Struct":{"a":1}}"#;
        let expected = E::Struct { a: 1 };
        assert_eq!(expected, from_str(j).unwrap());
    }
}
