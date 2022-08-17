// Integer container types for annotated serialization.
//
use num_traits::int::PrimInt;
use std::fmt;
use std::num::ParseIntError;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum Base {
    Bin = 2,
    Oct = 8,
    Dec = 10,
    Hex = 16,
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

    // Converts an `IntValue` to text with the requested base and output width.
    // The integer is represented with enough leading zeros to meet the output width.
    // The output width may be exceeded if the integer cannot fit within the
    // requested space.
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

    pub fn negate(self) -> Self {
        match self {
            IntValue::U8(v) => IntValue::I16(-(v as i16)),
            IntValue::U16(v) => IntValue::I32(-(v as i32)),
            IntValue::U32(v) => IntValue::I64(-(v as i64)),
            IntValue::U64(v) => IntValue::I128(-(v as i128)),
            IntValue::U128(v) => IntValue::I128(-(v as i128)),
            IntValue::I8(v) => IntValue::I8(-v),
            IntValue::I16(v) => IntValue::I16(-v),
            IntValue::I32(v) => IntValue::I32(-v),
            IntValue::I64(v) => IntValue::I64(-v),
            IntValue::I128(v) => IntValue::I128(-v),
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

    /// Returns whether the integer is within the legal range of json integers.
    pub fn is_legal_json(&self) -> bool {
        match self.value {
            IntValue::U64(v) => v < (1 << 53),
            IntValue::U128(v) => v < (1 << 53),
            IntValue::I64(v) => v > -(1 << 53) && v < (1 << 53),
            IntValue::I128(v) => v > -(1 << 53) && v < (1 << 53),
            _ => true,
        }
    }

    /// Returns the preferred base for expressing this integer.
    pub fn base(&self) -> Base {
        self.base
    }

    /// Formats the integer in the requested base, defaulting to the preferred base.
    pub fn format(&self, base: Option<&Base>) -> String {
        self.value.format(*base.unwrap_or(&Base::Dec), self.width)
    }

    fn strip_numeric_prefix<'a>(src: &'a str, ch: u8) -> &'a str {
        let lo = ['0', (ch | 0x20) as char];
        let up = ['0', (ch & !0x20) as char];
        if src.starts_with(&lo) || src.starts_with(&up) {
            &src[2..]
        } else {
            src
        }
    }
    fn detect_numeric_prefix<'a>(src: &'a str) -> (Base, &'a str) {
        let bytes = src.as_bytes();
        if src.len() >= 2 && bytes[0] == b'0' {
            match bytes[1] {
                b'b' | b'B' => (Base::Bin, &src[2..]),
                b'o' | b'O' => (Base::Oct, &src[2..]),
                b'x' | b'X' => (Base::Hex, &src[2..]),
                _ => (Base::Dec, src),
            }
        } else {
            (Base::Dec, src)
        }
    }

    /// Converts from a string into an integer value.
    /// - If the `radix` is 2, 8 or 16, the integer is parsed in that base.
    ///   The integer may start with one of the common prefixes `0x`, `0b`, or `0o`.
    ///
    /// - If `radix` is `0`, the base is inferred from the common integer
    ///   prefixes `0x`, `0b` and `0o`.  If there is no prefix, the base defaults
    ///   to base 10.
    pub fn from_str_radix(src: &str, radix: u32) -> Result<Int, ParseIntError> {
        let (negative, src) = if let Some(s) = src.strip_prefix('-') {
            (true, s)
        } else if let Some(s) = src.strip_prefix('+') {
            (false, s)
        } else {
            (false, src)
        };
        let (base, text) = match radix {
            2 => (Base::Bin, Self::strip_numeric_prefix(src, b'b')),
            8 => (Base::Oct, Self::strip_numeric_prefix(src, b'o')),
            16 => (Base::Hex, Self::strip_numeric_prefix(src, b'x')),
            10 => (Base::Dec, src),
            _ => Self::detect_numeric_prefix(src),
        };
        let value = IntValue::U128(u128::from_str_radix(text, base as u32)?);
        let value = if negative { value.negate() } else { value };
        Ok(Self::new_with_padding(value, base, text.len()))
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(Some(&self.base)))
    }
}

macro_rules! impl_from_int {
    ($t:ty) => {
        /// Consumes the `Int` converting to a primitive type.
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
        /// Converts the `Int` to a primitive type.
        impl From<&Int> for $t {
            fn from(val: &Int) -> Self {
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
impl_from_int!(f32);
impl_from_int!(f64);

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

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
    fn basic_parse() -> Result<()> {
        assert_eq!(u8::from(Int::from_str_radix("0b10", 0)?), 2);
        assert_eq!(u8::from(Int::from_str_radix("0o10", 0)?), 8);
        assert_eq!(u8::from(Int::from_str_radix("0x10", 0)?), 16);
        assert_eq!(u8::from(Int::from_str_radix("10", 0)?), 10);

        assert_eq!(i8::from(Int::from_str_radix("0b11111110", 0)?), -2);
        assert_eq!(i8::from(Int::from_str_radix("0o370", 0)?), -8);
        assert_eq!(i8::from(Int::from_str_radix("0xF0", 0)?), -16);
        assert_eq!(i8::from(Int::from_str_radix("-10", 0)?), -10);
        Ok(())
    }

    #[test]
    fn basic_roundtrip() -> Result<()> {
        assert_eq!(
            Int::from_str_radix("0x12345678", 0)?.to_string(),
            "0x12345678"
        );
        // Base and leading zeros are preserved.
        assert_eq!(Int::from_str_radix("0b0001", 0)?.to_string(), "0b0001");
        // Base-identifier and Hex capitalization are not preserved.
        assert_eq!(Int::from_str_radix("0Xab", 0)?.to_string(), "0xAB");
        Ok(())
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
