use crate::document::{Document, KeyValue, StrFormat};
use crate::error::Error;
use crate::integer::{Base, Int};
use once_cell::sync::OnceCell;
use std::collections::HashSet;
use std::fmt;

type Result<T> = std::result::Result<T, Error>;

pub enum Comment {
    None,
    Hash,
    SlashSlash,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Multiline {
    None,
    Json5,
    Hjson,
}

pub struct Json {
    document: Document,
    indent: usize,
    comment: Comment,
    bases: HashSet<Base>,
    literals: HashSet<Base>,
    strict_numeric_limits: bool,
    multiline: Multiline,
    bare_keys: bool,
    compact: bool,
}

impl Json {
    pub fn indent(mut self, i: usize) -> Self {
        self.indent = i;
        self
    }
    pub fn comment(mut self, c: Comment) -> Self {
        self.comment = c;
        self
    }
    pub fn bases(mut self, b: &[Base]) -> Self {
        for x in b {
            self.bases.insert(*x);
        }
        self
    }
    pub fn literals(mut self, b: &[Base]) -> Self {
        for x in b {
            self.bases.insert(*x);
            self.literals.insert(*x);
        }
        self
    }
    pub fn strict_numeric_limits(mut self, b: bool) -> Self {
        self.strict_numeric_limits = b;
        self
    }
    pub fn multiline(mut self, m: Multiline) -> Self {
        self.multiline = m;
        self
    }
    pub fn bare_keys(mut self, b: bool) -> Self {
        self.bare_keys = b;
        self
    }
    pub fn compact(mut self, b: bool) -> Self {
        self.compact = b;
        self
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut emitter = JsonEmitter {
            level: 0,
            indent: self.indent,
            comment: match self.comment {
                Comment::None => None,
                Comment::Hash => Some("#".to_string()),
                Comment::SlashSlash => Some("//".to_string()),
            },
            bases: self.bases.clone(),
            literals: self.literals.clone(),
            strict_numeric_limits: self.strict_numeric_limits,
            multiline: self.multiline,
            bare_keys: self.bare_keys,
            compact: self.compact,
        };
        emitter.emit_node(f, &self.document).map_err(|_| fmt::Error)
    }
}

impl Document {
    pub fn to_json(self) -> Json {
        Json {
            document: self,
            indent: 2,
            comment: Comment::None,
            bases: HashSet::from([Base::Dec]),
            literals: HashSet::from([Base::Dec]),
            strict_numeric_limits: true,
            multiline: Multiline::None,
            bare_keys: false,
            compact: false,
        }
    }

    pub fn to_json5(self) -> Json {
        self.to_json()
            .comment(Comment::SlashSlash)
            .literals(&[Base::Hex])
            .multiline(Multiline::Json5)
            .bare_keys(true)
    }

    pub fn to_hjson(self) -> Json {
        self.to_json()
            .comment(Comment::Hash)
            .multiline(Multiline::Hjson)
            .bare_keys(true)
    }
}

pub struct JsonEmitter {
    level: usize,
    indent: usize,
    comment: Option<String>,
    bases: HashSet<Base>,
    literals: HashSet<Base>,
    strict_numeric_limits: bool,
    multiline: Multiline,
    bare_keys: bool,
    compact: bool,
}

impl Default for JsonEmitter {
    fn default() -> Self {
        JsonEmitter {
            level: 0,
            indent: 2,
            comment: None,
            bases: HashSet::new(),
            literals: HashSet::new(),
            strict_numeric_limits: true,
            multiline: Multiline::None,
            bare_keys: false,
            compact: false,
        }
    }
}

impl JsonEmitter {
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
        self.level += 1;
        self.writeln(w, "[")?;
        self.emit_indent(w)?;
        for (i, value) in bytes.iter().enumerate() {
            if i > 0 {
                self.writeln(w, ",")?;
                self.emit_indent(w)?;
            }
            write!(w, "{}", value)?;
        }
        self.writeln(w, "")?;
        self.level -= 1;
        self.emit_indent(w)?;
        write!(w, "]")?;
        Ok(())
    }

    fn emit_sequence<W: fmt::Write>(&mut self, w: &mut W, sequence: &[Document]) -> Result<()> {
        self.level += 1;
        self.writeln(w, "[")?;
        self.emit_indent(w)?;
        for (i, value) in sequence.iter().enumerate() {
            if i > 0 {
                self.writeln(w, ",")?;
                self.emit_indent(w)?;
            }
            self.emit_node(w, value)?;
        }
        self.writeln(w, "")?;
        self.level -= 1;
        self.emit_indent(w)?;
        write!(w, "]")?;
        Ok(())
    }

    fn emit_mapping<W: fmt::Write>(&mut self, w: &mut W, mapping: &[KeyValue]) -> Result<()> {
        self.level += 1;
        self.writeln(w, "{")?;
        self.emit_indent(w)?;
        let mut comments = 0;
        for (i, KeyValue(key, value)) in mapping.iter().enumerate() {
            if i - comments > 0 {
                self.writeln(w, ",")?;
                self.emit_indent(w)?;
            }
            match key {
                Document::Comment(c) => {
                    // If the key is a comment, there's no useful value
                    self.emit_comment(w, c.as_str())?;
                    comments += 1;
                    continue;
                }
                Document::String(s, _) => {
                    if self.bare_keys && is_legal_bareword(s.as_str()) {
                        write!(w, "{}", s)?
                    } else {
                        write!(w, "\"{}\"", s)?
                    }
                }
                Document::Boolean(v) => write!(w, "\"{}\"", v)?,
                Document::Int(v) => write!(w, "\"{}\"", v)?,
                Document::Float(v) => write!(w, "\"{}\"", v)?,
                Document::Mapping(_) => return Err(Error::KeyTypeError("mapping")),
                Document::Sequence(_) => return Err(Error::KeyTypeError("sequence")),
                Document::Bytes(_) => return Err(Error::KeyTypeError("bytes")),
                Document::Compact(_) => return Err(Error::KeyTypeError("compact")),
                Document::Null => return Err(Error::KeyTypeError("null")),
            };
            write!(w, ": ")?;
            self.emit_node(w, value)?;
        }
        self.writeln(w, "")?;
        self.level -= 1;
        self.emit_indent(w)?;
        write!(w, "}}")?;
        Ok(())
    }

    fn emit_comment<W: fmt::Write>(&mut self, w: &mut W, comment: &str) -> Result<()> {
        if self.comment.is_none() || self.compact {
            return Ok(());
        }
        for line in comment.split('\n') {
            if line.is_empty() {
                writeln!(w, "{}", self.comment.as_ref().unwrap())?;
            } else {
                writeln!(w, "{} {}", self.comment.as_ref().unwrap(), line)?;
            }
            self.emit_indent(w)?;
        }
        Ok(())
    }

    fn emit_string<W: fmt::Write>(&mut self, w: &mut W, value: &str, f: StrFormat) -> Result<()> {
        if self.multiline != Multiline::None && f == StrFormat::Multiline {
            self.emit_string_multiline(w, value)
        } else {
            self.emit_string_strict(w, value)
        }
    }

    fn emit_string_strict<W: fmt::Write>(&mut self, w: &mut W, value: &str) -> Result<()> {
        write!(w, "\"")?;
        let bytes = value.as_bytes();
        let mut start = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }
            if start < i {
                write!(w, "{}", &value[start..i])?;
            }
            match escape {
                UU => write!(w, "\\u{:04x}", byte)?,
                _ => write!(w, "\\{}", byte as char)?,
            };
            start = i + 1;
        }
        if start != bytes.len() {
            write!(w, "{}", &value[start..])?;
        }
        write!(w, "\"")?;
        Ok(())
    }

    fn emit_string_multiline<W: fmt::Write>(&mut self, w: &mut W, value: &str) -> Result<()> {
        if self.multiline == Multiline::Hjson {
            writeln!(w)?;
            self.level += 1;
            self.emit_indent(w)?;
            writeln!(w, "'''")?;
            self.emit_indent(w)?;
        } else {
            write!(w, "\"")?;
        }
        let bytes = value.as_bytes();
        let mut start = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }
            if start < i {
                write!(w, "{}", &value[start..i])?;
            }
            match escape {
                UU => write!(w, "\\u{:04x}", byte)?,
                NN => match self.multiline {
                    Multiline::None => write!(w, "\\{}", byte as char)?,
                    Multiline::Json5 => writeln!(w, "\\")?,
                    Multiline::Hjson => {
                        writeln!(w)?;
                        self.emit_indent(w)?;
                    }
                },
                _ => write!(w, "\\{}", byte as char)?,
            };
            start = i + 1;
        }
        if start != bytes.len() {
            write!(w, "{}", &value[start..])?;
        }
        if self.multiline == Multiline::Hjson {
            writeln!(w)?;
            self.emit_indent(w)?;
            write!(w, "'''")?;
            self.level -= 1;
        } else {
            write!(w, "\"")?;
        }
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
        let b = i.base();
        let s = i.format(self.bases.get(&b));
        if self.strict_numeric_limits && !i.is_legal_json()
            || self.bases.get(&b).is_some() && self.literals.get(&b).is_none()
        {
            write!(w, "\"{}\"", s)?;
        } else {
            write!(w, "{}", s)?;
        }
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
        if self.compact {
            return Ok(());
        }
        let mut len = self.level * self.indent;
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

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
const ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

const SPACE: &str = "                                                                                                    ";

// More strict than javascript.
fn bad_identifier_char(ch: char) -> bool {
    match ch {
        '0'..='9' => false,
        'A'..='Z' => false,
        'a'..='z' => false,
        '_' => false,
        '$' => false,
        _ => true,
    }
}

fn is_reserved_word(word: &str) -> bool {
    static WORDS: OnceCell<HashSet<&str>> = OnceCell::new();
    let words = WORDS.get_or_init(|| {
        HashSet::from([
            "break",
            "do",
            "instanceof",
            "typeof",
            "case",
            "else",
            "new",
            "var",
            "catch",
            "finally",
            "return",
            "void",
            "continue",
            "for",
            "switch",
            "while",
            "debugger",
            "function",
            "this",
            "with",
            "default",
            "if",
            "throw",
            "",
            "delete",
            "in",
            "try",
            "class",
            "enum",
            "extends",
            "super",
            "const",
            "export",
            "import",
            "implements",
            "let",
            "private",
            "public",
            "yield",
            "interface",
            "package",
            "protected",
            "static",
            "null",
            "true",
            "false",
        ])
    });
    words.get(word).is_some()
}

fn is_legal_bareword(word: &str) -> bool {
    if word.len() == 0 {
        return false;
    }
    let ch = word.chars().nth(0).unwrap();
    !((ch >= '0' && ch <= '9') || word.contains(bad_identifier_char) || is_reserved_word(word))
}

#[cfg(test)]
mod test {
    use super::*;

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
        let c = comment("woohoo!").to_json();
        assert_eq!(c.to_string(), "");
        let c = comment("woohoo!").to_json5();
        assert_eq!(c.to_string(), "// woohoo!\n");
        let n = null().to_json();
        assert_eq!(n.to_string(), "null");
        let b = boolean(true).to_json();
        assert_eq!(b.to_string(), "true");
        // Plain integer
        let i = int(5).to_json();
        assert_eq!(i.to_string(), "5");
        // Integer wants to be hex, but hex isn't allowed.
        let i = hex(15).to_json();
        assert_eq!(i.to_string(), "15");
        // Integer wants to be hex, hex is allowed, but not as a literal.
        let i = hex(16).to_json().bases(&[Base::Hex]);
        assert_eq!(i.to_string(), "\"0x10\"");
        // Integer wants to be hex, hex literals allowed.
        let i = hex(16).to_json5();
        assert_eq!(i.to_string(), "0x10");
        let s = string("hello").to_json();
        assert_eq!(s.to_string(), "\"hello\"");
        let f = float(3.14159).to_json();
        assert_eq!(f.to_string(), "3.14159");
    }

    #[test]
    fn basic_list() {
        let expect = r#"[
  5,
  10,
  15,
  "foo"
]"#;

        let list = Document::Sequence(vec![int(5), int(10), int(15), string("foo")]).to_json();
        assert_eq!(list.to_string(), expect);
    }

    #[test]
    fn basic_map() {
        let expect = r#"{
  "a": 5,
  "b": 10,
  "c": 15,
  "true": "foo"
}"#;
        let map = Document::Mapping(vec![
            kv("a", int(5)),
            kv("b", int(10)),
            kv("c", int(15)),
            kv("true", string("foo")),
        ])
        .to_json();
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn basic_map5() {
        let expect = r#"{
  a: 5,
  b: 10,
  c: 0xF,
  "true": "foo"
}"#;
        let map = Document::Mapping(vec![
            kv("a", int(5)),
            kv("b", int(10)),
            kv("c", hex(15)),
            kv("true", string("foo")),
        ])
        .to_json5();
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn compact_map5() {
        let expect = r#"{a: 5, b: 10, c: 0xF, "true": "foo"}"#;
        let map = Document::Mapping(vec![
            kv("a", int(5)),
            kv("b", int(10)),
            kv("c", hex(15)),
            kv("true", string("foo")),
        ])
        .to_json5()
        .compact(true);
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn mixed_map5() {
        let expect = r#"{
  gameplay: {prg: [0, 0x8000]},
  overworld: {prg: [1, 0x8000]},
  palaces: {prg: [4, 0x8000]},
  title: {prg: [5, 0x8000]},
  music: {prg: [6, 0x8000]},
  reset: {prg: [-1, 0xFFFA]}
}"#;
        let map = Document::Mapping(vec![
            kv("gameplay", nes_address("prg", 0, 0x8000)),
            kv("overworld", nes_address("prg", 1, 0x8000)),
            kv("palaces", nes_address("prg", 4, 0x8000)),
            kv("title", nes_address("prg", 5, 0x8000)),
            kv("music", nes_address("prg", 6, 0x8000)),
            kv("reset", nes_address("prg", -1, 0xFFFA)),
        ])
        .to_json5();
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn demo_map5() {
        let expect = r#"{
  // comments
  unquoted: "and you can quote me on that",
  singleQuotes: "not really, though",
  lineBreaks: "Look, Mom! \
No \\n's!",
  hexadecimal: 0xDECAF,
  "leadingDecimal(not)": 0.8675309,
  "andTrailing(not)": 8675309,
  "positiveSign(not)": 1,
  "trailingComma(not)": [
    "in objects",
    "or arrays"
  ],
  backwardsCompatible: "with JSON"
}"#;
        let map = Document::Mapping(vec![
            KeyValue(comment("comments"), null()),
            kv("unquoted", string("and you can quote me on that")),
            kv("singleQuotes", string("not really, though")),
            kv("lineBreaks", multistr("Look, Mom! \nNo \\n's!")),
            kv("hexadecimal", hex(0xdecaf)),
            kv("leadingDecimal(not)", float(0.8675309)),
            kv("andTrailing(not)", float(8675309.0)),
            kv("positiveSign(not)", int(1)),
            kv(
                "trailingComma(not)",
                Document::Sequence(vec![string("in objects"), string("or arrays")]),
            ),
            kv("backwardsCompatible", string("with JSON")),
        ])
        .to_json5();
        assert_eq!(map.to_string(), expect);
    }

    #[test]
    fn demo_maph() {
        let expect = r#"{
  # comments
  unquoted: "and you can quote me on that",
  singleQuotes: "not really, though",
  lineBreaks: 
    '''
    Look, Mom!
    No \\n's!
    ''',
  hexadecimal: 912559,
  "leadingDecimal(not)": 0.8675309,
  "andTrailing(not)": 8675309,
  "positiveSign(not)": 1,
  "trailingComma(not)": [
    "in objects",
    "or arrays"
  ],
  backwardsCompatible: "with JSON"
}"#;
        let map = Document::Mapping(vec![
            KeyValue(comment("comments"), null()),
            kv("unquoted", string("and you can quote me on that")),
            kv("singleQuotes", string("not really, though")),
            kv("lineBreaks", multistr("Look, Mom!\nNo \\n's!")),
            kv("hexadecimal", hex(0xdecaf)),
            kv("leadingDecimal(not)", float(0.8675309)),
            kv("andTrailing(not)", float(8675309.0)),
            kv("positiveSign(not)", int(1)),
            kv(
                "trailingComma(not)",
                Document::Sequence(vec![string("in objects"), string("or arrays")]),
            ),
            kv("backwardsCompatible", string("with JSON")),
        ])
        .to_hjson();
        println!("{}", map);
        assert_eq!(map.to_string(), expect);
    }
}
