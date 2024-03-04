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
}

pub mod private {

    use super::Annotate;
    pub use inventory;
    use once_cell::sync::OnceCell;
    use std::collections::HashMap;
    use std::sync::Mutex;

    type CastFn = unsafe fn(object: *const ()) -> &'static dyn Annotate;
    /// An `Annotator` holds the name of a struct and casting function that can
    /// cast the named type into a `dyn Annotate` reference.
    /// See the safety comment for `Annotator::new`.
    pub struct Annotator {
        name: &'static str,
        into_annotate: CastFn,
    }
    inventory::collect!(Annotator);

    static ANNOTATORS: OnceCell<Mutex<HashMap<&'static str, CastFn>>> = OnceCell::new();
    impl Annotator {
        /// Creates a new `Annotator` for the named type.
        /// Safety: `name` must be the name of a type that implements the
        /// `Annotate` trait and `into_annotate` must cast the provided
        /// pointer into a `&dyn Annotate` reference for that type.
        pub const unsafe fn new(name: &'static str, into_annotate: CastFn) -> Self {
            Annotator {
                name,
                into_annotate,
            }
        }
        fn lookup(typename: &str) -> Option<CastFn> {
            let annotators = ANNOTATORS
                .get_or_init(|| {
                    let mut casts = HashMap::new();
                    for a in inventory::iter::<Annotator> {
                        let previous = casts.insert(a.name, a.into_annotate);
                        if previous.is_some() {
                            panic!("Annotator typename {:?} duplicated.", a.name);
                        }
                    }
                    Mutex::new(casts)
                })
                .lock()
                .unwrap();
            annotators.get(typename).cloned()
        }

        /// Cast the `object` into a `dyn Annotate` reference.
        pub fn cast<'a>(typename: &str, object: &AnyPointer<'a>) -> Option<&'a dyn Annotate> {
            if object.is_null() {
                None
            } else {
                let a = Self::lookup(typename).map(|cast| unsafe {
                    // Safety: If we found the type, its safe to cast it to
                    // dyn Annotate.  Cast the object and use transmute to
                    // re-attach the lifetime 'a to the result.
                    std::mem::transmute(cast(object.ptr))
                });
                a
            }
        }
    }

    #[derive(Clone)]
    pub struct AnyPointer<'a> {
        ptr: *const (),
        lifetime: std::marker::PhantomData<&'a ()>,
    }

    impl AnyPointer<'_> {
        pub fn new<'a, T>(object: &'a T) -> AnyPointer<'a>
        where
            T: ?Sized,
        {
            AnyPointer {
                ptr: object as *const T as *const (),
                lifetime: std::marker::PhantomData,
            }
        }

        pub fn null() -> AnyPointer<'static> {
            AnyPointer {
                ptr: std::ptr::null(),
                lifetime: std::marker::PhantomData,
            }
        }

        pub fn is_null(&self) -> bool {
            self.ptr.is_null()
        }
    }
}
