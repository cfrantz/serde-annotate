pub use annotate_derive::*;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Mutex;

pub enum MemberId<'a> {
    Name(&'a str),
    Index(u32),
    Variant,
}

pub enum Format {
    Block,
    Binary,
    Decimal,
    Hex,
    Octal,
    Compact,
}

pub trait Annotate {
    fn format(&self, variant: Option<&str>, field: &MemberId) -> Option<Format>;
    fn comment(&self, variant: Option<&str>, field: &MemberId) -> Option<String>;
}

type IdFn = fn() -> usize;
type CastFn = unsafe fn(*const ()) -> &'static dyn Annotate;

pub struct AnnotateType {
    pub id: IdFn,
    pub cast: CastFn,
}
inventory::collect!(AnnotateType);

impl AnnotateType {
    pub fn type_id<T>() -> usize
    where
        T: ?Sized,
    {
        // Just like https://github.com/rust-lang/rust/issues/41875#issuecomment-317292888
        // We monomorphize on T and then cast the function pointer address of
        // the monomorphized `AnnotateType::type_id` function to an
        // integer identifier.
        Self::type_id::<T> as usize
    }

    pub unsafe fn cast<T>(ptr: *const ()) -> &'static dyn Annotate
    where
        T: 'static + Annotate,
    {
        // Cast a generic pointer back to a reference to T and return a
        // dyn reference to the Annotate trait.
        &*(ptr as *const T)
    }

    fn lookup(id: usize) -> Option<CastFn> {
        static TYPEMAP: OnceCell<Mutex<HashMap<usize, CastFn>>> = OnceCell::new();
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

    pub fn get<'a, T>(object: &'a T) -> Option<&'a dyn Annotate>
    where
        T: ?Sized,
    {
        // Get the type-id of `object` can cast it to `Annotate` if we can.
        let id = Self::type_id::<T>();
        Self::lookup(id).map(|cast| unsafe {
            // Shorten the lifetime to 'a, as the dyn Annotate reference is
            // really a reinterpretation of `object`, which has lifetime 'a.
            std::mem::transmute::<&'static dyn Annotate, &'a dyn Annotate>(cast(
                object as *const T as *const (),
            ))
        })
    }
}
