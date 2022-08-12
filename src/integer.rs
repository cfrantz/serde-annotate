// Integer container types for annotated serialization.
//
use num_traits::int::PrimInt;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Base {
    Bin,
    Dec,
    Hex,
    Oct,
}

#[derive(Clone, Debug)]
pub enum IntValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
}

macro_rules! impl_from_primitive {
    ($t:ty, $f:ident) => {
        impl From<$t> for IntValue {
            fn from(val: $t) -> Self {
                IntValue::$f(val)
            }
        }
    };
}

impl_from_primitive!(u8, U8);
impl_from_primitive!(u16, U16);
impl_from_primitive!(u32, U32);
impl_from_primitive!(u64, U64);
impl_from_primitive!(u128, U128);
impl_from_primitive!(i8, I8);
impl_from_primitive!(i16, I16);
impl_from_primitive!(i32, I32);
impl_from_primitive!(i64, I64);
impl_from_primitive!(i128, I128);

impl IntValue {
    const HEX: &'static [u8; 16] = b"0123456789ABCDEF";

    fn convert<T: PrimInt + ToString>(mut v: T, base: Base, mut width: usize) -> String {
        let shift = match base {
            Base::Bin => 1,
            Base::Oct => 3,
            Base::Dec => return v.to_string(),
            Base::Hex => 4,
        };
        const BITS: usize = 128;
        if width > BITS {
            let bits = std::mem::size_of_val(&v) * 8;
            width = match base {
                Base::Bin => bits,
                Base::Oct => (bits + 2) / 3,
                Base::Hex => (bits + 3) / 4,
                Base::Dec => unreachable!(),
            };
        }
        let mask = T::one().unsigned_shl(shift) - T::one();
        let mut buffer = [0u8; 2 + BITS];
        let mut i = 2 + BITS;
        loop {
            i -= 1;
            buffer[i] = Self::HEX[v.bitand(mask).to_usize().unwrap()];
            width = width.saturating_sub(1);
            v = v.unsigned_shr(shift);
            if v.is_zero() && width == 0 {
                break;
            }
        }
        match base {
            Base::Bin => {
                i -= 2;
                buffer[i] = b'0';
                buffer[i + 1] = b'b';
            }
            Base::Oct => {
                i -= 2;
                buffer[i] = b'0';
                buffer[i + 1] = b'o';
            }
            Base::Hex => {
                i -= 2;
                buffer[i] = b'0';
                buffer[i + 1] = b'x';
            }
            Base::Dec => unreachable!(),
        }
        // Utf8Error is impossible here.
        std::str::from_utf8(&buffer[i..]).unwrap().to_string()
    }

    pub fn format(&self, base: Base, bitwidth: usize) -> String {
        match self {
            IntValue::U8(v) => Self::convert(*v, base, bitwidth),
            IntValue::U16(v) => Self::convert(*v, base, bitwidth),
            IntValue::U32(v) => Self::convert(*v, base, bitwidth),
            IntValue::U64(v) => Self::convert(*v, base, bitwidth),
            IntValue::U128(v) => Self::convert(*v, base, bitwidth),
            IntValue::I8(v) => Self::convert(*v, base, bitwidth),
            IntValue::I16(v) => Self::convert(*v, base, bitwidth),
            IntValue::I32(v) => Self::convert(*v, base, bitwidth),
            IntValue::I64(v) => Self::convert(*v, base, bitwidth),
            IntValue::I128(v) => Self::convert(*v, base, bitwidth),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Int {
    value: IntValue,
    base: Base,
    width: usize,
}

impl Int {
    /// Creates an `Int` that will display with a minimum of `width` characters,
    /// zero-padded as needed.
    pub fn new_with_padding<T: Into<IntValue>>(v: T, base: Base, width: usize) -> Int {
        Int {
            value: v.into(),
            base,
            width,
        }
    }
    /// Creates an `Int` that will display with no zero padding.
    pub fn new<T: Into<IntValue>>(v: T, base: Base) -> Int {
        Self::new_with_padding(v, base, 0)
    }
    /// Creates an `Int` that will display zero padding appropriate for `T`.
    pub fn new_padded<T: Into<IntValue>>(v: T, base: Base) -> Int {
        Self::new_with_padding(v, base, usize::MAX)
    }

    pub fn is_legal_json(&self) -> bool {
        match self.value {
            IntValue::U64(v) => v < (1 << 53),
            IntValue::U128(v) => v < (1 << 53),
            IntValue::I64(v) => v > -(1 << 53) && v < (1 << 53),
            IntValue::I128(v) => v > -(1 << 53) && v < (1 << 53),
            _ => true,
        }
    }

    pub fn base(&self) -> Base {
        self.base
    }

    pub fn format(&self, base: Option<&Base>) -> String {
        self.value.format(*base.unwrap_or(&Base::Dec), self.width)
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(Some(&self.base)))
    }
}

macro_rules! impl_from_int {
    ($t:ty) => {
        impl From<Int> for $t {
            fn from(val: Int) -> Self {
                match val.value {
                    IntValue::U8(v) => v as $t,
                    IntValue::U16(v) => v as $t,
                    IntValue::U32(v) => v as $t,
                    IntValue::U64(v) => v as $t,
                    IntValue::U128(v) => v as $t,
                    IntValue::I8(v) => v as $t,
                    IntValue::I16(v) => v as $t,
                    IntValue::I32(v) => v as $t,
                    IntValue::I64(v) => v as $t,
                    IntValue::I128(v) => v as $t,
                }
            }
        }
    };
}

impl_from_int!(u8);
impl_from_int!(u16);
impl_from_int!(u32);
impl_from_int!(u64);
impl_from_int!(u128);
impl_from_int!(i8);
impl_from_int!(i16);
impl_from_int!(i32);
impl_from_int!(i64);
impl_from_int!(i128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_conversions() {
        assert_eq!(Int::new(2u8, Base::Bin).to_string(), "0b10");
        assert_eq!(Int::new(8u8, Base::Oct).to_string(), "0o10");
        assert_eq!(Int::new(10u8, Base::Dec).to_string(), "10");
        assert_eq!(Int::new(16u8, Base::Hex).to_string(), "0x10");

        assert_eq!(Int::new(-2i8, Base::Bin).to_string(), "0b11111110");
        assert_eq!(Int::new(-8i8, Base::Oct).to_string(), "0o370");
        assert_eq!(Int::new(-10i8, Base::Dec).to_string(), "-10");
        assert_eq!(Int::new(-16i8, Base::Hex).to_string(), "0xF0");
    }

    #[test]
    fn basic_padding() {
        assert_eq!(Int::new_padded(2u8, Base::Bin).to_string(), "0b00000010");
        assert_eq!(Int::new_padded(8u16, Base::Oct).to_string(), "0o000010");
        assert_eq!(Int::new_padded(10u32, Base::Dec).to_string(), "10");
        assert_eq!(Int::new_padded(16u32, Base::Hex).to_string(), "0x00000010");

        assert_eq!(Int::new_padded(255u128, Base::Bin).to_string(), "0b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000011111111");
        assert_eq!(
            Int::new_padded(255u128, Base::Hex).to_string(),
            "0x000000000000000000000000000000FF"
        );
        assert_eq!(
            Int::new_padded(-256i128, Base::Hex).to_string(),
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00"
        );
    }

    #[test]
    fn exceeds_padding() {
        assert_eq!(
            Int::new_with_padding(16u8, Base::Bin, 1).to_string(),
            "0b10000"
        );
        assert_eq!(
            Int::new_with_padding(16u8, Base::Oct, 1).to_string(),
            "0o20"
        );
        assert_eq!(Int::new_with_padding(16u8, Base::Dec, 1).to_string(), "16");
        assert_eq!(
            Int::new_with_padding(65536u32, Base::Hex, 1).to_string(),
            "0x10000"
        );
    }
}
