pub use annotate_derive::*;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Mutex;

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
}

#[cfg(not(feature = "type_by_name"))]
pub type IdType = usize;
#[cfg(feature = "type_by_name")]
pub type IdType = &'static str;

type IdFn = fn() -> IdType;
type CastFn = unsafe fn(*const ()) -> &'static dyn Annotate;

pub struct AnnotateType {
    pub id: IdFn,
    pub cast: CastFn,
}
inventory::collect!(AnnotateType);

impl AnnotateType {
    #[cfg(not(feature = "type_by_name"))]
    pub fn type_id<T>() -> IdType
    where
        T: ?Sized,
    {
        // We can't use std::any::TypeId here because we don't want to
        // limit T to 'static.
        //
        // Just like https://github.com/rust-lang/rust/issues/41875#issuecomment-317292888
        // We monomorphize on T and then cast the function pointer address of
        // the monomorphized `AnnotateType::type_id` function to an
        // integer identifier.
        //
        // This can't be relied upon because the compiler might optimize this away.
        // In practice, it seems to work.
        Self::type_id::<T> as usize
    }

    #[cfg(feature = "type_by_name")]
    pub fn type_id<T>() -> IdType
    where
        T: ?Sized,
    {
        // Instead of using the hacky "type_id" monomorphization trick, we use
        // the type name given back by the standard library.
        //
        // This can't be relied upon because the standard library makes no
        // guarantees about the returned string. In practice, it seems to work.
        std::any::type_name::<T>()
    }

    // This is unsafe because we give the returned trait object
    // a 'static lifetime.  We use transmute later to shorten the
    // lifetime down to the known lifetime.
    pub unsafe fn cast<T>(ptr: *const ()) -> &'static dyn Annotate
    where
        T: 'static + Annotate,
    {
        // Cast a generic pointer back to a reference to T and return a
        // dyn reference to the Annotate trait.
        &*(ptr as *const T)
    }

    fn lookup(id: IdType) -> Option<CastFn> {
        static TYPEMAP: OnceCell<Mutex<HashMap<IdType, CastFn>>> = OnceCell::new();
        let typemap = TYPEMAP
            .get_or_init(|| {
                let mut types = HashMap::new();
                for annotate in inventory::iter::<AnnotateType> {
                    types.insert((annotate.id)(), annotate.cast);
                }
                Mutex::new(types)
            })
            .lock()
            .unwrap();
        typemap.get(&id).cloned()
    }

    #[cfg(not(feature = "erased"))]
    pub fn get<'a, T>(object: &'a T) -> Option<&'a dyn Annotate>
    where
        T: ?Sized + serde::Serialize,
    {
        // Get the type-id of `object` and cast it to `Annotate` if we can.
        let id = Self::type_id::<T>();
        Self::lookup(id).map(|cast| unsafe {
            // Shorten the lifetime to 'a, as the dyn Annotate reference is
            // really a fat pointer to `object`, which has lifetime 'a.
            std::mem::transmute::<&'static dyn Annotate, &'a dyn Annotate>(cast(
                object as *const T as *const (),
            ))
        })
    }

    #[cfg(feature = "erased")]
    pub fn get<'a, T>(object: &'a T) -> Option<&'a dyn Annotate>
    where
        T: ?Sized + erased_serde::Serialize,
    {
        // Get the type-id of `object` and cast it to `Annotate` if we can.
        let id = object.type_name();
        Self::lookup(id).map(|cast| unsafe {
            // Shorten the lifetime to 'a, as the dyn Annotate reference is
            // really a fat pointer to `object`, which has lifetime 'a.
            std::mem::transmute::<&'static dyn Annotate, &'a dyn Annotate>(cast(
                object as *const T as *const (),
            ))
        })
    }
}
