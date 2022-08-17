use std::fmt::Write;
use crate::document::BytesFormat;
use crate::error::Error;
use regex::RegexBuilder;

const HEX: &[u8; 16] = b"0123456789abcdef";

// Emit bytes as a hex string (e.g. "cafef00d0badc0de").
fn hexstr(data: &[u8]) -> String {
    let mut s = String::with_capacity(2 * data.len());
    for byte in data {
        s.push(HEX[(byte >> 4) as usize] as char);
        s.push(HEX[(byte & 0x0F) as usize] as char);
    }
    s
}

// Emit bytes as a hexdump in the style of `hexdump -vC`.
fn hexdump(data: &[u8]) -> String {
    // Hexdump always emits a full line of output (78 chars plus newline)
    // regardless of the input length.  Round the input length up to the next
    // multple of 16 while calculating the output length.
    let mut s = String::with_capacity((data.len() + 15) * 79 / 16);
    for (i, chunk) in data.chunks(16).enumerate() {
        if i > 0 {
            s.push('\n');
        }
        write!(s, "{:08x}", i * 16).unwrap();
        let mut buf = [b'.'; 16];
        let mut space = 51;
        for (j, &byte) in chunk.iter().enumerate() {
            if j % 8 == 0 {
                s.push(' ');
                space -= 1;
            }
            s.push(' ');
            s.push(HEX[(byte >> 4) as usize] as char);
            s.push(HEX[(byte & 0x0F) as usize] as char);
            space -= 3;
            buf[j] = match byte {
                0x20..=0x7f => byte,
                _ => b'.',
            };
        }
        // Utf8Error is impossible here because all of the codepoints
        // inside `buf` are ASCII.
        let chars = std::str::from_utf8(&buf[..chunk.len()]).unwrap();
        write!(s, "{0:>1$} |{2}|", " ", space, chars).unwrap();
    }
    s
}

// Emit bytes as a hexdump in the style of `xxd -g<grouping>``.
fn xxd(data: &[u8], grouping: usize) -> String {
    // Xxd always emits a full line of output regardless of the input length.
    // In smallest grouping mode (-g1), each line is 75 chars plus a newline.
    // Round the input length up to the next multple of 16 while calculating
    // the output length.
    let mut s = String::with_capacity((data.len() + 15) * 76 / 16);
    for (i, chunk) in data.chunks(16).enumerate() {
        if i > 0 {
            s.push('\n');
        }
        write!(s, "{:08x}:", i * 16).unwrap();
        let mut buf = [b'.'; 16];
        let mut space = (16 / grouping) * (grouping * 2 + 1) + 1;
        for (j, &byte) in chunk.iter().enumerate() {
            if j % grouping == 0 {
                s.push(' ');
                space -= 1;
            }
            s.push(HEX[(byte >> 4) as usize] as char);
            s.push(HEX[(byte & 0x0F) as usize] as char);
            space -= 2;
            buf[j] = match byte {
                0x20..=0x7f => byte,
                _ => b'.',
            };
        }
        // Utf8Error is impossible here because all of the codepoints
        // inside `buf` are ASCII.
        let chars = std::str::from_utf8(&buf[..chunk.len()]).unwrap();
        write!(s, "{0:>1$} {2}", " ", space, chars).unwrap();
    }
    s
}

/// Convers a byte buffer to a hexadecimal string in `format`.
pub fn to_string(data: &[u8], format: BytesFormat) -> Option<String> {
    match format {
        BytesFormat::HexStr => Some(hexstr(data)),
        BytesFormat::Hexdump => Some(hexdump(data)),
        // By default, `xxd` emits outputs with grouping 2.
        BytesFormat::Xxd => Some(xxd(data, 2)),
        _ => None,
    }
}

// Translate an ASCII byte into its hex numerical value.
fn unhex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

// Given a hex string, parse hex bytes and append them to `vec`.
fn from_hex(text: &str, vec: &mut Vec<u8>) -> Result<(), Error> {
    let mut it = text.bytes().filter_map(unhex);
    while let Some(a) = it.next() {
        if let Some(b) = it.next() {
            vec.push(a << 4 | b);
        } else {
            return Err(Error::HexdumpError(
                "odd number of hex input characters".into(),
            ));
        }
    }
    Ok(())
}

/// Parses a hexdump string in a variety of forms, returning the resulting bytes.
pub fn from_str(text: &str) -> Result<Vec<u8>, Error> {
    // Detects `xxd -g<n>` formats.
    let xxd = RegexBuilder::new(r"^[[:xdigit:]]{8}:\s+((?:[[:xdigit:]]{2,}\s)+)\s+.{1,16}$")
        .multi_line(true)
        .build()
        .unwrap();
    // Detects `hexdump -vC`
    let hexdump =
        RegexBuilder::new(r"^[[:xdigit:]]{8}\s+((?:[[:xdigit:]]{2}\s+?){1,16})\s+\|.*\|$")
            .multi_line(true)
            .build()
            .unwrap();
    // Detects a simple hex string with optional whitespace.
    let hexstr = RegexBuilder::new(r"(?:0[xX])?((?:[[:xdigit:]]{2}\s*)+)")
        .multi_line(false)
        .build()
        .unwrap();

    let mut res = Vec::new();
    let captures = if xxd.is_match(text) {
        xxd.captures_iter(text)
    } else if hexdump.is_match(text) {
        hexdump.captures_iter(text)
    } else if hexstr.is_match(text) {
        hexstr.captures_iter(text)
    } else {
        return Err(Error::HexdumpError("unrecognized format".into()));
    };
    for c in captures {
        from_hex(c.get(1).unwrap().as_str(), &mut res)?;
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_hexstr() -> Result<()> {
        let buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
        let res = hexstr(&buf);
        assert_eq!(res, "000102030405060708090a0b0c0d0e0f1011");
        Ok(())
    }

    const TEST_STR: &str = "The quick brown fox jumped over the lazy dog!";

    // Output from `hexdump -vC ...`
    const HEXDUMP_C: &str = "\
00000000  54 68 65 20 71 75 69 63  6b 20 62 72 6f 77 6e 20  |The quick brown |\n\
00000010  66 6f 78 20 6a 75 6d 70  65 64 20 6f 76 65 72 20  |fox jumped over |\n\
00000020  74 68 65 20 6c 61 7a 79  20 64 6f 67 21           |the lazy dog!|";

    // Output from `xxd -g<n> ...` where n = {1,2,4,8}
    const XXD_G1: &str = "\
00000000: 54 68 65 20 71 75 69 63 6b 20 62 72 6f 77 6e 20  The quick brown \n\
00000010: 66 6f 78 20 6a 75 6d 70 65 64 20 6f 76 65 72 20  fox jumped over \n\
00000020: 74 68 65 20 6c 61 7a 79 20 64 6f 67 21           the lazy dog!";

    const XXD_G2: &str = "\
00000000: 5468 6520 7175 6963 6b20 6272 6f77 6e20  The quick brown \n\
00000010: 666f 7820 6a75 6d70 6564 206f 7665 7220  fox jumped over \n\
00000020: 7468 6520 6c61 7a79 2064 6f67 21         the lazy dog!";

    const XXD_G4: &str = "\
00000000: 54686520 71756963 6b206272 6f776e20  The quick brown \n\
00000010: 666f7820 6a756d70 6564206f 76657220  fox jumped over \n\
00000020: 74686520 6c617a79 20646f67 21        the lazy dog!";

    const XXD_G8: &str = "\
00000000: 5468652071756963 6b2062726f776e20  The quick brown \n\
00000010: 666f78206a756d70 6564206f76657220  fox jumped over \n\
00000020: 746865206c617a79 20646f6721        the lazy dog!";

    const XXD: [&str; 4] = [XXD_G1, XXD_G2, XXD_G4, XXD_G8];

    #[test]
    fn test_hexdump() -> Result<()> {
        let buf = TEST_STR;
        let res = hexdump(buf.as_bytes());
        assert_eq!(res, HEXDUMP_C);
        Ok(())
    }

    #[test]
    fn test_xxd() -> Result<()> {
        let buf = TEST_STR;
        for n in 0..XXD.len() {
            let res = xxd(buf.as_bytes(), 1 << n);
            assert_eq!(res, XXD[n]);
        }
        Ok(())
    }

    #[test]
    fn test_from_hexstr() -> Result<()> {
        let buf = "5468652071756963\n6b2062726f776e20";
        let res = from_str(buf)?;
        let s = std::str::from_utf8(&res)?;
        assert_eq!(s, "The quick brown ");
        Ok(())
    }

    #[test]
    fn test_from_hexdump() -> Result<()> {
        let res = from_str(HEXDUMP_C)?;
        let s = std::str::from_utf8(&res)?;
        assert_eq!(s, TEST_STR);
        Ok(())
    }

    #[test]
    fn test_from_xxd() -> Result<()> {
        for n in 0..XXD.len() {
            let res = from_str(XXD[n])?;
            let s = std::str::from_utf8(&res)?;
            assert_eq!(s, TEST_STR);
        }
        Ok(())
    }
}
