use crate::document::{Document, KeyValue, StrFormat};
use crate::error::Error;
use crate::integer::Int;
use std::fmt;

type Result<T> = std::result::Result<T, Error>;

pub struct Yaml {
    document: Document,
    indent: usize,
    compact: bool,
    header: bool,
}

impl Yaml {
    pub fn indent(mut self, i: usize) -> Self {
        self.indent = i;
        self
    }
    pub fn compact(mut self, b: bool) -> Self {
        self.compact = b;
        self
    }
    pub fn header(mut self, b: bool) -> Self {
        self.header = b;
        self
    }

}

impl fmt::Display for Yaml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut emitter = YamlEmitter {
            level: -1,
            indent: self.indent,
            compact: self.compact,
        };
        if self.header {
            writeln!(f, "---")?;
        }
        emitter.emit_node(f, &self.document).map_err(|_| fmt::Error)
    }
}

impl Document {
    pub fn to_yaml(self) -> Yaml {
        Yaml {
            document: self,
            indent: 2,
            compact: false,
            header: true,
        }
    }
}

pub struct YamlEmitter {
    level: isize,
    indent: usize,
    compact: bool,
}

impl Default for YamlEmitter {
    fn default() -> Self {
        YamlEmitter {
            level: -1,
            indent: 2,
            compact: false,
        }
    }
}

impl YamlEmitter {
    fn emit_node<W: fmt::Write>(&mut self, w: &mut W, node: &Document) -> Result<()> {
        match node {
            Document::Comment(c) => self.emit_comment(w, c.as_str()),
            Document::String(v, f) => self.emit_string(w, v.as_str(), *f),
            Document::Boolean(v) => self.emit_boolean(w, *v),
            Document::Int(v) => self.emit_int(w, v),
            Document::Float(v) => self.emit_float(w, *v),
            Document::Mapping(m) => self.emit_mapping(w, m),
            Document::Sequence(s) => self.emit_sequence(w, s),
            Document::Bytes(v) => self.emit_bytes(w, v),
            Document::Null => self.emit_null(w),
            Document::Compact(d) => self.emit_compact(w, d),
        }
    }

    fn emit_compact<W: fmt::Write>(&mut self, w: &mut W, node: &Document) -> Result<()> {
        let compact = self.compact;
        self.compact = true;
        self.emit_node(w, node)?;
        self.compact = compact;
        Ok(())
    }

    fn emit_bytes<W: fmt::Write>(&mut self, w: &mut W, bytes: &[u8]) -> Result<()> {
        self.writeln(w, "[")?;
        self.emit_indent(w)?;
        for (i, chunk) in bytes.chunks(16).enumerate() {
            if i>0 { self.writeln(w, "")?; }
            for b in chunk {
                write!(w, "0x{:02X},", b)?;
            }
        }
        self.writeln(w, "]")?;
        self.emit_indent(w)?;
        Ok(())
    }


    fn emit_helper<W: fmt::Write>(&mut self, w: &mut W, prefix: &str, value: &Document) -> Result<()> {
        match value {
            Document::Sequence(v) => {
                if self.compact || v.is_empty() {
                    write!(w, "{} ", prefix)?;
                } else {
                    writeln!(w, "{}", prefix)?;
                    self.emit_indent_extra(w, 1)?
                }
            },
            Document::Mapping(v) => {
                if self.compact || v.is_empty() {
                    write!(w, "{} ", prefix)?;
                } else {
                    writeln!(w, "{}", prefix)?;
                    self.emit_indent_extra(w, 1)?
                }
            },
            _ => write!(w, "{} ", prefix)?,
        };
        self.emit_node(w, value)
    }

    fn emit_sequence<W: fmt::Write>(&mut self, w: &mut W, sequence: &[Document]) -> Result<()> {
        if self.compact || sequence.is_empty() {
            write!(w, "[")?;
            for (i, v) in sequence.iter().enumerate() {
                if i>0 { write!(w, ", ")?; }
                self.emit_node(w, v)?;
            }
            write!(w, "]")?;
        } else {
            self.level += 1;
            for (i, v) in sequence.iter().enumerate() {
                if i>0 { writeln!(w)?; self.emit_indent(w)? }
                self.emit_helper(w, "-", v)?;
            }
            self.level -= 1;
        }
        Ok(())
    }

    fn emit_mapping<W: fmt::Write>(&mut self, w: &mut W, mapping: &[KeyValue]) -> Result<()> {
        if self.compact || mapping.is_empty() {
            write!(w, "{{")?;
        } else {
            self.level += 1;
        }
        let mut skip = true;
        for KeyValue(key, value) in mapping.iter() {
            if !skip {
                if self.compact {
                    write!(w, ", ")?;
                } else {
                    writeln!(w)?; self.emit_indent(w)?;
                }
            }
            if let Document::Comment(c) = &key {
                self.emit_comment(w, c.as_str())?;
                skip = true;
                continue;
            }
            self.emit_node(w, &key)?;
            self.emit_helper(w, ":", value)?;
            skip = false;
        }
        if self.compact || mapping.is_empty() {
            write!(w, "}}")?;
        } else {
            self.level -= 1;
        }
        Ok(())
    }

    fn emit_comment<W: fmt::Write>(&mut self, w: &mut W, comment: &str) -> Result<()> {
        if !self.compact {
        for line in comment.split('\n') {
            if line.is_empty() {
                writeln!(w, "#")?;
            } else {
                writeln!(w, "# {}", line)?;
            }
            self.emit_indent(w)?;
        }
        }
        Ok(())
    }

    fn emit_string<W: fmt::Write>(&mut self, w: &mut W, value: &str, f: StrFormat) -> Result<()> {
        match f {
            StrFormat::Multiline => self.emit_string_multiline(w, value)?,
            StrFormat::Quoted => escape_str(w, value, true)?,
            StrFormat::Standard => escape_str(w, value, need_quotes(value))?,
        }
        Ok(())
    }

    fn emit_string_multiline<W: fmt::Write>(&mut self, w: &mut W, mut value: &str) -> Result<()> {
        if value.ends_with('\n') {
            write!(w, "|+")?;
            value = &value[..value.len()-1];
        } else {
            write!(w, "|-")?;
        }
        self.level += 1;
        for line in value.split('\n') {
            writeln!(w)?;
            self.emit_indent(w)?;
            escape_str(w, line, false)?;
        }
        self.level -= 1;
        Ok(())
    }

    fn emit_boolean<W: fmt::Write>(&mut self, w: &mut W, b: bool) -> Result<()> {
        if b {
            write!(w, "true")?;
        } else {
            write!(w, "false")?;
        }
        Ok(())
    }

    fn emit_int<W: fmt::Write>(&mut self, w: &mut W, i: &Int) -> Result<()> {
        write!(w, "{}", i)?;
        Ok(())
    }

    fn emit_float<W: fmt::Write>(&mut self, w: &mut W, f: f64) -> Result<()> {
        write!(w, "{}", f)?;
        Ok(())
    }

    fn emit_null<W: fmt::Write>(&mut self, w: &mut W) -> Result<()> {
        write!(w, "null")?;
        Ok(())
    }

    fn emit_indent<W: fmt::Write>(&mut self, w: &mut W) -> Result<()> {
        self.emit_indent_extra(w, 0)
    }

    fn emit_indent_extra<W: fmt::Write>(&mut self, w: &mut W, extra: isize) -> Result<()> {
        let extra = self.level + extra;
        if self.compact || extra < 0 {
            return Ok(());
        }
        let mut len = (extra as usize) * self.indent;
        while len > 0 {
            let chunk = std::cmp::min(len, SPACE.len());
            write!(w, "{}", &SPACE[..chunk])?;
            len -= chunk;
        }
        Ok(())
    }

    fn writeln<W: fmt::Write>(&mut self, w: &mut W, s: &str) -> Result<()> {
        if self.compact {
            match s {
                "," => write!(w, ", ")?,
                _ => write!(w, "{}", s)?,
            };
        } else {
            writeln!(w, "{}", s)?;
        }
        Ok(())
    }
}

const SPACE: &str = "                                                                                                    ";
// From yaml-rust:
fn escape_str<W: fmt::Write>(wr: &mut W, v: &str, quoted: bool) -> std::result::Result<(), fmt::Error> {
    if quoted {
        wr.write_str("\"")?;
    }

    let mut start = 0;
    for (i, byte) in v.bytes().enumerate() {
        let escaped = match byte {
            b'"' if quoted => "\\\"",
            b'\\' => "\\\\",
            b'\x00' => "\\u0000",
            b'\x01' => "\\u0001",
            b'\x02' => "\\u0002",
            b'\x03' => "\\u0003",
            b'\x04' => "\\u0004",
            b'\x05' => "\\u0005",
            b'\x06' => "\\u0006",
            b'\x07' => "\\u0007",
            b'\x08' => "\\b",
            b'\t' => "\\t",
            b'\n' => "\\n",
            b'\x0b' => "\\u000b",
            b'\x0c' => "\\f",
            b'\r' => "\\r",
            b'\x0e' => "\\u000e",
            b'\x0f' => "\\u000f",
            b'\x10' => "\\u0010",
            b'\x11' => "\\u0011",
            b'\x12' => "\\u0012",
            b'\x13' => "\\u0013",
            b'\x14' => "\\u0014",
            b'\x15' => "\\u0015",
            b'\x16' => "\\u0016",
            b'\x17' => "\\u0017",
            b'\x18' => "\\u0018",
            b'\x19' => "\\u0019",
            b'\x1a' => "\\u001a",
            b'\x1b' => "\\u001b",
            b'\x1c' => "\\u001c",
            b'\x1d' => "\\u001d",
            b'\x1e' => "\\u001e",
            b'\x1f' => "\\u001f",
            b'\x7f' => "\\u007f",
            _ => continue,
        };
        if start < i {
            wr.write_str(&v[start..i])?;
        }
        wr.write_str(escaped)?;
        start = i + 1;
    }

    if start != v.len() {
        wr.write_str(&v[start..])?;
    }

    if quoted {
        wr.write_str("\"")?;
    }
    Ok(())
}

// From yaml-rust:
// Check if the string requires quoting.
// Strings starting with any of the following characters must be quoted.
// :, &, *, ?, |, -, <, >, =, !, %, @
// Strings containing any of the following characters must be quoted.
// {, }, [, ], ,, #, `
//
// If the string contains any of the following control characters, it must be escaped with double quotes:
// \0, \x01, \x02, \x03, \x04, \x05, \x06, \a, \b, \t, \n, \v, \f, \r, \x0e, \x0f, \x10, \x11, \x12, \x13, \x14, \x15, \x16, \x17, \x18, \x19, \x1a, \e, \x1c, \x1d, \x1e, \x1f, \N, \_, \L, \P
//
// Finally, there are other cases when the strings must be quoted, no matter if you're using single or double quotes:
// * When the string is true or false (otherwise, it would be treated as a boolean value);
// * When the string is null or ~ (otherwise, it would be considered as a null value);
// * When the string looks like a number, such as integers (e.g. 2, 14, etc.), floats (e.g. 2.6, 14.9) and exponential numbers (e.g. 12e7, etc.) (otherwise, it would be treated as a numeric value);
// * When the string looks like a date (e.g. 2014-12-31) (otherwise it would be automatically converted into a Unix timestamp).
fn need_quotes(string: &str) -> bool {
    fn need_quotes_spaces(string: &str) -> bool {
        string.starts_with(' ') || string.ends_with(' ')
    }

    string == ""
        || need_quotes_spaces(string)
        || string.starts_with(|character: char| match character {
            '&' | '*' | '?' | '|' | '-' | '<' | '>' | '=' | '!' | '%' | '@' => true,
            _ => false,
        })
        || string.contains(|character: char| match character {
            ':'
            | '{'
            | '}'
            | '['
            | ']'
            | ','
            | '#'
            | '`'
            | '\"'
            | '\''
            | '\\'
            | '\0'..='\x06'
            | '\t'
            | '\n'
            | '\r'
            | '\x0e'..='\x1a'
            | '\x1c'..='\x1f' => true,
            _ => false,
        })
        || [
            // http://yaml.org/type/bool.html
            // Note: 'y', 'Y', 'n', 'N', is not quoted deliberately, as in libyaml. PyYAML also parse
            // them as string, not booleans, although it is violating the YAML 1.1 specification.
            // See https://github.com/dtolnay/serde-yaml/pull/83#discussion_r152628088.
            "yes", "Yes", "YES", "no", "No", "NO", "True", "TRUE", "true", "False", "FALSE",
            "false", "on", "On", "ON", "off", "Off", "OFF",
            // http://yaml.org/type/null.html
            "null", "Null", "NULL", "~",
        ]
        .contains(&string)
        || string.starts_with('.')
        || string.starts_with("0x")
        || string.parse::<i64>().is_ok()
        || string.parse::<f64>().is_ok()
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::integer::Base;

    fn int(v: i32) -> Document {
        Document::Int(Int::new(v, Base::Dec))
    }
    fn hex(v: u32) -> Document {
        Document::Int(Int::new(v, Base::Hex))
    }
    fn float(v: f64) -> Document {
        Document::Float(v)
    }
    fn boolean(v: bool) -> Document {
        Document::Boolean(v)
    }
    fn null() -> Document {
        Document::Null
    }
    fn string(v: &str) -> Document {
        Document::String(v.to_string(), StrFormat::Standard)
    }
    fn multistr(v: &str) -> Document {
        Document::String(v.to_string(), StrFormat::Multiline)
    }
    fn comment(v: &str) -> Document {
        Document::Comment(v.to_string())
    }
    fn kv(k: &str, v: Document) -> KeyValue {
        KeyValue(string(k), v)
    }
    fn nes_address(seg: &str, bank: i32, addr: u32) -> Document {
        Document::Compact(
            Document::Mapping(vec![kv(
                seg,
                Document::Sequence(vec![int(bank), hex(addr)]),
            )])
            .into(),
        )
    }

    #[test]
    fn basic_document() {
        let c = comment("woohoo!").to_yaml().header(false);
        assert_eq!(c.to_string(), "# woohoo!\n");
        let n = null().to_yaml().header(false);
        assert_eq!(n.to_string(), "null");
        let b = boolean(true).to_yaml().header(false);
        assert_eq!(b.to_string(), "true");
        let i = int(5).to_yaml().header(false);
        assert_eq!(i.to_string(), "5");
        let i = hex(16).to_yaml().header(false);
        assert_eq!(i.to_string(), "0x10");
        let s = string("hello").to_yaml().header(false);
        assert_eq!(s.to_string(), "hello");
        let f = float(3.14159).to_yaml().header(false);
        assert_eq!(f.to_string(), "3.14159");
    }

    #[test]
    fn basic_list() {
        let expect = r#"---
- 5
- 10
- 15
- foo"#;

        let list = Document::Sequence(vec![int(5), int(10), int(15), string("foo")]).to_yaml();
        assert_eq!(list.to_string(), expect);
    }

    #[test]
    fn basic_map() {
        let expect = r#"---
a: 5
b: 10
c: 15
"true": foo"#;
        let map = Document::Mapping(vec![
            kv("a", int(5)),
            kv("b", int(10)),
            kv("c", int(15)),
            kv("true", string("foo")),
        ])
        .to_yaml();
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn basic_map_hex() {
        let expect = r#"---
a: 5
b: 10
c: 0xF
"true": foo"#;
        let map = Document::Mapping(vec![
            kv("a", int(5)),
            kv("b", int(10)),
            kv("c", hex(15)),
            kv("true", string("foo")),
        ])
        .to_yaml();
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn compact_map_hex() {
        let expect = r#"---
{a: 5, b: 10, c: 0xF, "true": foo}"#;
        let map = Document::Mapping(vec![
            kv("a", int(5)),
            kv("b", int(10)),
            kv("c", hex(15)),
            kv("true", string("foo")),
        ])
        .to_yaml()
        .compact(true);
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn mixed_map5() {
        let expect = r#"---
gameplay: {prg: [0, 0x8000]}
overworld: {prg: [1, 0x8000]}
palaces: {prg: [4, 0x8000]}
title: {prg: [5, 0x8000]}
music: {prg: [6, 0x8000]}
reset: {prg: [-1, 0xFFFA]}"#;
        let map = Document::Mapping(vec![
            kv("gameplay", nes_address("prg", 0, 0x8000)),
            kv("overworld", nes_address("prg", 1, 0x8000)),
            kv("palaces", nes_address("prg", 4, 0x8000)),
            kv("title", nes_address("prg", 5, 0x8000)),
            kv("music", nes_address("prg", 6, 0x8000)),
            kv("reset", nes_address("prg", -1, 0xFFFA)),
        ])
        .to_yaml();
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn demo_map_json5() {
        let expect = r#"---
# comments
unquoted: and you can quote me on that
singleQuotes: "not really, though"
lineBreaks: |-
  Look, Mom!
  No \\n's!
hexadecimal: 0xDECAF
# more comments
leadingDecimal(not): 0.8675309
andTrailing(not): 8675309
positiveSign(not): 1
trailingComma(not):
  - in objects
  - or arrays
backwardsCompatible: with JSON"#;
        let map = Document::Mapping(vec![
            KeyValue(comment("comments"), null()),
            kv("unquoted", string("and you can quote me on that")),
            kv("singleQuotes", string("not really, though")),
            kv("lineBreaks", multistr("Look, Mom!\nNo \\n's!")),
            kv("hexadecimal", hex(0xdecaf)),
            KeyValue(comment("more comments"), null()),
            kv("leadingDecimal(not)", float(0.8675309)),
            kv("andTrailing(not)", float(8675309.0)),
            kv("positiveSign(not)", int(1)),
            kv(
                "trailingComma(not)",
                Document::Sequence(vec![string("in objects"), string("or arrays")]),
            ),
            kv("backwardsCompatible", string("with JSON")),
        ])
        .to_yaml();
        println!("{}", map);
        assert_eq!(map.to_string(), expect);
    }
}
