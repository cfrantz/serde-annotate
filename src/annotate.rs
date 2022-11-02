use std::fmt;

use crate::{AnnotatedSerializer, Deserializer, Document, Error};

/// Specifies the formatting options to use when serializing.
pub enum Format {
    /// Format a string in block/multiline style.
    Block,
    /// Format an integer as binary.
    Binary,
    /// Format an integer as decimal.
    Decimal,
    /// Format an integer as hexadecimal.
    Hex,
    /// Format an integer as octal.
    Octal,
    /// Format an aggregate in compact mode.
    Compact,
    /// Format a bytes object as a hex string.
    HexStr,
    /// Format a bytes object as hexdump (e.g. `hexdump -vC <file>`).
    Hexdump,
    /// Format a bytes object as xxd (e.g. `xxd <file>`).
    Xxd,
}

/// Identifies a field or variant member of a struct/enum.
pub enum MemberId<'a> {
    Name(&'a str),
    Index(u32),
    Variant,
}

/// Trait implemented on structs to inform the serializer about formatting
/// options and comments.
pub trait Annotate {
    fn format(&self, variant: Option<&str>, field: &MemberId) -> Option<Format>;
    fn comment(&self, variant: Option<&str>, field: &MemberId) -> Option<String>;
    fn as_annotate(&self) -> Option<&dyn Annotate>;
    fn thunk_serialize(&self, serializer: &mut AnnotatedSerializer) -> Result<Document, Error>;
}

/// The default implementation of Annotate returns no comments or annotations and
/// cannot return the trait object.
impl<T: ?Sized + serde::Serialize> Annotate for T {
    default fn format(&self, _variant: Option<&str>, _field: &MemberId) -> Option<Format> {
        None
    }
    default fn comment(&self, _variant: Option<&str>, _field: &MemberId) -> Option<String> {
        None
    }
    default fn as_annotate(&self) -> Option<&dyn Annotate> {
        None
    }
    default fn thunk_serialize(
        &self,
        serializer: &mut AnnotatedSerializer,
    ) -> Result<Document, Error> {
        self.serialize(serializer)
    }
}

// Serde explicitly implements Serialize on &T where T: Serialize.  This
// causes min_specialization to select the default implementation for &T
// even though there is a specialized implementation available for T.
//
// The annotate_derive crate uses this macro to create the additional
// specializations needed.
#[macro_export]
macro_rules! annotate_ref {
    ($ty:ty) => {
        $crate::__annotate_ref!(&$ty);
    };
}

#[macro_export]
macro_rules! __annotate_ref {
    ($ty:ty) => {
        impl Annotate for $ty {
            fn format(&self, variant: Option<&str>, field: &MemberId) -> Option<Format> {
                (**self).format(variant, field)
            }
            fn comment(&self, variant: Option<&str>, field: &MemberId) -> Option<String> {
                (**self).comment(variant, field)
            }
            fn as_annotate(&self) -> Option<&dyn Annotate> {
                (**self).as_annotate()
            }
        }
    };
}

// We use a private trait to identify whether the Serializer passed to
// various functions is our Serializer.
pub(crate) unsafe trait IsSerializer {
    fn is_serde_annotate(&self) -> bool;
}

unsafe impl<T: serde::Serializer> IsSerializer for T {
    default fn is_serde_annotate(&self) -> bool {
        false
    }
}

unsafe impl<'a> IsSerializer for &mut AnnotatedSerializer<'a> {
    fn is_serde_annotate(&self) -> bool {
        true
    }
}

// This marker trait is to avoid specifying lifetimes in the default
// implementation.  When I specify lifetimes in the default impl, the
// compiler complains that the specialized impl repeats parameter `'de`.
trait _IsDeserializer {}
impl<'de, T: serde::Deserializer<'de>> _IsDeserializer for T {}

// We use a private trait to identify whether the Deserializer passed to
// various functions is our Deserializer.
pub(crate) unsafe trait IsDeserializer {
    fn is_serde_annotate(&self) -> bool;
}

unsafe impl<T: _IsDeserializer> IsDeserializer for T {
    default fn is_serde_annotate(&self) -> bool {
        false
    }
}

unsafe impl<'de> IsDeserializer for &mut Deserializer<'de> {
    fn is_serde_annotate(&self) -> bool {
        true
    }
}

// Dime-store type erasure: Implement serde::Serialize on the Annotate trait object
// so one can pass the trait objects into `serde_annotate::serialize()` and get
// serialized objects out.  I'm doing this because:
// - Without some sort of `TypeId` support for non-`'static` types, its impossible
//   to properly determine if they implement `Annotate`.
// - Without some `TypeId` rememberance added into `erased-serde`, its impossible
//   to properly determine if the type-erased object implemented `Annotate`.
//
// The strategy here (the dime-store part) is to assume the serializer will be
// AnnotatedSerializer and just force the types with `transmute`.
impl serde::Serialize for dyn Annotate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !serializer.is_serde_annotate() {
            panic!(
                "Expected to be called by AnnotatedSerializer, not {:?}",
                std::any::type_name::<S>()
            );
        }
        unsafe {
            // If `serializer` is the correct type, then we can transmute the
            // reference into `&mut AnnotatedSerializer` and forget the prior reference.
            let szr: &mut AnnotatedSerializer = std::mem::transmute_copy(&serializer);
            std::mem::forget(serializer);
            let r = self.thunk_serialize(szr);
            // Similarly, if the `serializer` was the correct type, we can assume the
            // return type will be correct, and thus the transmute is a no-op... Actually,
            // its a simple copy because `transmute` can't be sure that
            // `Result<Document, Error>` is the same size as whatever
            // `Result<S::Ok, S::Error>` happens to be.  They _will_ be the same size
            // (indeed the same type) because only `AnnotatedSerializer` is permitted to
            // call this function and it wants `Result<Document, Error>` returned).
            let result = std::mem::transmute_copy(&r);
            std::mem::forget(r);
            result
        }
    }
}

impl fmt::Debug for dyn Annotate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn Annotate({:p})", self)
    }
}
