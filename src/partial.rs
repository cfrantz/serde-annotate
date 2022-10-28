use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::annotate::{IsDeserializer, IsSerializer};
use crate::Deserializer as AnnotatedDeserializer;
use crate::{Document, Error};

impl Serialize for Document {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_serde_annotate() {
            // If `serializer` is the correct type, then we can clone the
            // Document node and return it.
            let r: Result<Document, Error> = Ok(self.clone());
            let result = unsafe {
                // We have to transmute because the we can't determine at compile
                // time that `Result<Document, Error>` is the same type as
                // `Result<S::Ok, S::Error>`.  If the serializer is
                // `AnnotatedSerializer`, then it must be the same.
                std::mem::transmute_copy(&r)
            };
            std::mem::forget(r);
            result
        } else {
            Err(serde::ser::Error::custom("Serializing document nodes is only supported with serde_annotate::AnnotatedSerializer"))
        }
    }
}

impl<'de> Deserialize<'de> for Document {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_serde_annotate() {
            unsafe {
                // If the deserializer is ours, then we can simply clone the
                // deserializer's document node.
                let dsz: &AnnotatedDeserializer = std::mem::transmute_copy(&deserializer);
                std::mem::forget(deserializer);
                Ok(dsz.doc.clone())
            }
        } else {
            Err(serde::de::Error::custom(
                "Deserializing document nodes is only supported with serde_annotate::Deserializer",
            ))
        }
    }
}
