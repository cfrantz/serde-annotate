use pest::error::Error as PestError;
use pest::iterators::{Pair, Pairs};
use pest::Parser as P;
use pest::Position;
use pest_derive::Parser;
use std::cell::RefCell;

use crate::document::{CommentFormat, Document, StrFormat};
use crate::error::Error;
use crate::integer::Int;

#[derive(Default)]
struct Inner {
    lines: Vec<usize>,
}

/// `Relax` is a permissive JSON parser that permits many common extensions to
/// JSON documents including comments, alternate integer bases, multiline
/// strings and relaxed handling of commas in aggregates.
///
/// The `Relax` parser is configurable and can allow or disallow each of these
/// extensions.  The default `Relax` parser is maximally permissive.
#[derive(Parser)]
#[grammar = "relax.pest"]
pub struct Relax {
    inner: RefCell<Inner>,
    pub comma_trailing: bool,
    pub comma_optional: bool,
    pub number_bin: bool,
    pub number_hex: bool,
    pub number_oct: bool,
    pub number_plus: bool,
    pub number_lax_dec_point: bool,
    pub string_single_quote: bool,
    pub string_unquoted: bool,
    pub string_ident: bool,
    pub string_json5_multiline: bool,
    pub string_hjson_multiline: bool,
    pub comment_slash: bool,
    pub comment_hash: bool,
    pub comment_block: bool,
}

pub(crate) type ParseError = PestError<Rule>;

impl Default for Relax {
    /// Returns a maximally permissive json parser.
    fn default() -> Self {
        Relax {
            inner: Default::default(),
            comma_trailing: true,
            comma_optional: true,
            number_bin: true,
            number_hex: true,
            number_oct: true,
            number_plus: true,
            number_lax_dec_point: true,
            string_single_quote: true,
            string_unquoted: true,
            string_ident: true,
            string_json5_multiline: true,
            string_hjson_multiline: true,
            comment_slash: true,
            comment_hash: true,
            comment_block: true,
        }
    }
}

impl Relax {
    /// Creates a strict json parser.
    pub fn json() -> Self {
        Self {
            comma_trailing: false,
            comma_optional: false,
            number_bin: false,
            number_hex: false,
            number_oct: false,
            number_plus: false,
            number_lax_dec_point: false,
            string_single_quote: false,
            string_unquoted: false,
            string_ident: false,
            string_json5_multiline: false,
            string_hjson_multiline: false,
            comment_slash: false,
            comment_hash: false,
            comment_block: false,
            ..Self::default()
        }
    }

    /// Creates a json5 parser.
    pub fn json5() -> Self {
        Self {
            comma_optional: false,
            string_unquoted: false,
            string_hjson_multiline: false,
            comment_hash: false,
            number_bin: false,
            number_oct: false,
            ..Self::default()
        }
    }

    /// Creates a hjson parser.
    pub fn hjson() -> Self {
        Self {
            string_json5_multiline: false,
            number_bin: false,
            number_hex: false,
            number_oct: false,
            number_plus: false,
            number_lax_dec_point: false,
            ..Self::default()
        }
    }

    /// Parses a string into a `Document`.
    pub fn from_str(&self, text: &str) -> Result<Document, Error> {
        // Iterate over the input text and remember the line breaks. Since we use
        // positioning information to infer which comments belong with which json
        // items, caching the line-number information speeds up parsing
        // quite a bit.
        let mut inner = Inner::default();
        inner.lines.push(0);
        for (i, ch) in text.char_indices() {
            if ch == '\n' {
                inner.lines.push(i);
            }
        }
        inner.lines.push(usize::MAX);
        self.inner.replace(inner);
        let json = Relax::parse(Rule::text, text)?.next().unwrap();
        self.handle_pair(json)
    }

    fn line_col(&self, pos: usize) -> (usize, usize) {
        let inner = self.inner.borrow();
        let line = match inner.lines.binary_search(&pos) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let col = pos - inner.lines[line];
        (line, col)
    }

    fn unhex(ch: char) -> u32 {
        match ch {
            '0'..='9' => (ch as u8 - b'0') as u32,
            'A'..='F' => (ch as u8 - b'A' + 10) as u32,
            'a'..='f' => (ch as u8 - b'a' + 10) as u32,
            _ => unreachable!(),
        }
    }

    fn unescape(text: &str) -> Result<String, Error> {
        let mut s = String::with_capacity(text.len());
        let mut it = text.chars();
        while let Some(ch) = it.next() {
            if ch == '\\' {
                let ch = it.next().unwrap();
                let decoded = match ch {
                    '"' => '"',
                    '/' => '/',
                    '\\' => '\\',
                    '\'' => '\'',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\n' => '\n', // json5 multi-line string.
                    'u' => {
                        let mut v = 0;
                        v = (v << 4) | Self::unhex(it.next().unwrap());
                        v = (v << 4) | Self::unhex(it.next().unwrap());
                        v = (v << 4) | Self::unhex(it.next().unwrap());
                        v = (v << 4) | Self::unhex(it.next().unwrap());
                        char::try_from(v)?
                    }
                    'x' => {
                        let mut v = 0;
                        v = (v << 4) | Self::unhex(it.next().unwrap());
                        v = (v << 4) | Self::unhex(it.next().unwrap());
                        char::try_from(v)?
                    }
                    _ => return Err(Error::EscapeError(ch)),
                };
                s.push(decoded);
            } else {
                s.push(ch);
            }
        }
        Ok(s)
    }

    fn from_str_radix(text: &str, radix: u32) -> Result<Document, Error> {
        match Int::from_str_radix(text, radix) {
            Ok(val) => Ok(Document::Int(val)),
            Err(_) => Ok(Document::String(text.into(), StrFormat::Standard)),
        }
    }

    fn handle_number(&self, pair: Pair<Rule>) -> Result<Document, Error> {
        let text = pair.as_str();
        let t = if let Some(t) = text.strip_prefix('+') {
            Self::syntax_error(!self.number_plus, "leading `+`", pair.as_span().start_pos())?;
            t
        } else if let Some(t) = text.strip_prefix('-') {
            t
        } else {
            text
        };
        if t.starts_with("0x") || t.starts_with("0X") {
            // Hexadecimal integer.
            Self::syntax_error(
                !self.number_hex,
                "hexadecimal literal",
                pair.as_span().start_pos(),
            )?;
            Self::from_str_radix(text, 16)
        } else if t.starts_with("0b") || t.starts_with("0B") {
            // Binary integer.
            Self::syntax_error(
                !self.number_bin,
                "binary literal",
                pair.as_span().start_pos(),
            )?;
            return Self::from_str_radix(text, 2);
        } else if t.starts_with("0o") || t.starts_with("0O") {
            // Octal integer.
            Self::syntax_error(
                !self.number_oct,
                "octal literal",
                pair.as_span().start_pos(),
            )?;
            return Self::from_str_radix(text, 8);
        } else if t.contains('.')
            || t.contains('e')
            || t.contains('E')
            || t == "NaN"
            || t == "Infinity"
        {
            // Floating point number.
            Self::syntax_error(
                !self.number_lax_dec_point && (t.starts_with('.') || t.ends_with('.')),
                "bad float literal",
                pair.as_span().start_pos(),
            )?;
            return Ok(Document::Float(text.parse().unwrap()));
        } else {
            // Decimal integer.
            return Self::from_str_radix(text, 10);
        }
    }

    fn handle_kvpair(&self, pairs: &mut Pairs<Rule>) -> Result<(Document, bool), Error> {
        let mut k = usize::MAX;
        let mut v = usize::MAX;
        let mut kv = vec![];
        let mut comma = false;
        while let Some(pair) = pairs.peek() {
            let rule = pair.as_rule();
            if rule == Rule::comma {
                comma = true;
                let _ = pairs.next();
                continue;
            }
            let (line, _) = self.line_col(pair.as_span().start());
            if rule == Rule::COMMENT {
                if v != usize::MAX {
                    if v == line {
                        // Comment on the same line as the value,
                        // keep the comment.
                    } else {
                        // Comment is not on the same line as the value,
                        // so exit the loop; the comment belongs to the
                        // next value.
                        break;
                    }
                } else {
                    // Comment before the value, keep the comment.
                }
            } else if k == usize::MAX {
                // If the pair isn't a comment or comma, and we haven't seen
                // the key, then it must be the key.
                // Keep it.
                k = line;
            } else if v == usize::MAX {
                // If the pair isn't a comment or comma, and we haven't seen
                // the value, then it must be the value.
                // Keep it.
                v = line;
            } else {
                // If the pair is a not a comment or comma and we've seen both
                // the key and value, it must be part of the next kvpair.
                // Exit the loop.
                break;
            }
            kv.push(self.handle_pair(pair)?);
            // Advance the iterator.
            let _ = pairs.next();
        }
        Ok((Document::Fragment(kv), comma))
    }

    fn handle_array_elem(&self, pairs: &mut Pairs<Rule>) -> Result<(Document, bool), Error> {
        let mut i = usize::MAX;
        let mut item = vec![];
        let mut comma = false;
        let mut saw_value = false;
        while let Some(pair) = pairs.peek() {
            let rule = pair.as_rule();
            if rule == Rule::comma {
                let _ = pairs.next();
                comma = true;
                continue;
            }
            let (line, _) = self.line_col(pair.as_span().start());
            if rule == Rule::COMMENT {
                if saw_value {
                    if i == line {
                        // Comment is on the same line as the value,
                        // keep the comment.
                    } else {
                        // Comment is not on the same line as the value,
                        // so exit the loop; the comment belongs to the
                        // next value.
                        break;
                    }
                } else {
                    // Comment is before the value, keep the comment.
                }
            } else if !saw_value {
                // If the pair isn't a comment or comma, it must be a value.
                // Keep the value.
                i = line;
                saw_value = true;
            } else {
                // If the pair is a value, but we've already seen a value,
                // its the next value.  Exit the loop.
                break;
            }
            item.push(self.handle_pair(pair)?);
            let _ = pairs.next();
        }
        if item.len() == 1 && item[0].comment().is_none() {
            Ok((item.pop().unwrap(), comma))
        } else {
            Ok((Document::Fragment(item), comma))
        }
    }

    fn strip_leading_prefix<'a>(lines: &[&'a str], prefix: char) -> Vec<&'a str> {
        let plen = lines.iter().fold(usize::MAX, |acc, s| {
            if s.is_empty() {
                acc
            } else {
                let plen = s.len() - s.trim_start_matches(prefix).len();
                std::cmp::min(acc, plen)
            }
        });
        lines
            .iter()
            .map(|s| if s.is_empty() { s } else { s.split_at(plen).1 })
            .collect::<Vec<_>>()
    }

    fn syntax_error(err: bool, msg: &str, pos: Position) -> Result<(), Error> {
        if err {
            let (ln, col) = pos.line_col();
            Err(Error::SyntaxError(
                msg.into(),
                ln,
                col,
                pos.line_of().trim_end().into(),
                "^",
            ))
        } else {
            Ok(())
        }
    }

    fn handle_comment(&self, pair: Pair<Rule>) -> Result<Document, Error> {
        let comment = pair.as_str();
        if let Some(c) = comment.strip_prefix("/*") {
            Self::syntax_error(
                !self.comment_block,
                "block comment",
                pair.as_span().start_pos(),
            )?;
            let c = c.strip_suffix("*/").unwrap().trim_end();
            let lines = c.split('\n').map(str::trim).collect::<Vec<_>>();
            let lines = Self::strip_leading_prefix(&lines, '*');
            let lines = Self::strip_leading_prefix(&lines, ' ');
            let start = if lines.first().map(|s| s.is_empty()) == Some(true) {
                1
            } else {
                0
            };
            let c = lines[start..].join("\n");
            Ok(Document::Comment(c, CommentFormat::Block))
        } else if comment.starts_with("//") {
            Self::syntax_error(
                !self.comment_slash,
                "slash comment",
                pair.as_span().start_pos(),
            )?;
            let lines = comment.split('\n').map(str::trim).collect::<Vec<_>>();
            let lines = Self::strip_leading_prefix(&lines, '/');
            let lines = Self::strip_leading_prefix(&lines, ' ');
            let end = lines.len()
                - if lines.last().map(|s| s.is_empty()) == Some(true) {
                    1
                } else {
                    0
                };
            let c = lines[..end].join("\n");
            Ok(Document::Comment(c, CommentFormat::SlashSlash))
        } else if comment.starts_with('#') {
            Self::syntax_error(
                !self.comment_hash,
                "hash comment",
                pair.as_span().start_pos(),
            )?;
            let lines = comment.split('\n').map(str::trim).collect::<Vec<_>>();
            let lines = Self::strip_leading_prefix(&lines, '#');
            let lines = Self::strip_leading_prefix(&lines, ' ');
            let end = lines.len()
                - if lines.last().map(|s| s.is_empty()) == Some(true) {
                    1
                } else {
                    0
                };
            let c = lines[..end].join("\n");
            Ok(Document::Comment(c, CommentFormat::Hash))
        } else {
            Err(Error::Unknown(comment.into()))
        }
    }

    fn handle_string(&self, pair: Pair<Rule>) -> Result<Document, Error> {
        let s = pair.as_str();
        if s.starts_with("'''") {
            Self::syntax_error(
                !self.string_hjson_multiline,
                "unexpected hjson multiline string",
                pair.as_span().start_pos(),
            )?;
            let s = &s[3..(s.len() - 3)].trim();
            let (_, column) = self.line_col(pair.as_span().start());
            let split = column - 1;
            let mut value = Vec::new();
            for line in s.split('\n') {
                if line.len() < split {
                    value.push(line);
                } else {
                    let (space, _) = line.split_at(split);
                    let space = split - space.trim().len();
                    let (_, text) = line.split_at(space);
                    value.push(text);
                }
            }
            Ok(Document::String(value.join("\n"), StrFormat::Multiline))
        } else if s.starts_with('\'') || s.starts_with('"') {
            Self::syntax_error(
                !self.string_single_quote && s.starts_with('\''),
                "single quote",
                pair.as_span().start_pos(),
            )?;
            let s = &s[1..(s.len() - 1)];
            let json5_line_cont = s.contains("\\\r\n")
                || s.contains("\\\r")
                || s.contains("\\\n")
                || s.contains("\\\u{2028}")
                || s.contains("\\\u{2029}");
            Self::syntax_error(
                !self.string_json5_multiline && json5_line_cont,
                "unexpected end of line",
                pair.as_span().start_pos(),
            )?;
            let format = if json5_line_cont {
                StrFormat::Multiline
            } else {
                StrFormat::Standard
            };
            Ok(Document::String(Self::unescape(s)?, format))
        } else {
            Self::syntax_error(
                !self.string_unquoted,
                "missing quotes",
                pair.as_span().start_pos(),
            )?;
            Ok(Document::String(s.trim().into(), StrFormat::Unquoted))
        }
    }

    fn handle_pair(&self, pair: Pair<Rule>) -> Result<Document, Error> {
        match pair.as_rule() {
            Rule::null => Ok(Document::Null),
            Rule::boolean => Ok(Document::Boolean(pair.as_str().parse().unwrap())),
            Rule::string => self.handle_string(pair),
            Rule::hjson_key => {
                Self::syntax_error(
                    !self.string_ident,
                    "missing quotes",
                    pair.as_span().start_pos(),
                )?;
                Ok(Document::String(pair.as_str().into(), StrFormat::Unquoted))
            }
            Rule::identifier => {
                Self::syntax_error(
                    !self.string_ident,
                    "missing quotes",
                    pair.as_span().start_pos(),
                )?;
                // TODO: add StrFormat::Unquoted
                Ok(Document::String(pair.as_str().into(), StrFormat::Unquoted))
            }
            Rule::number => self.handle_number(pair),
            Rule::object => {
                let mut pairs = pair.into_inner();
                let mut npair = pairs.peek();
                let mut kvs = Vec::new();
                let mut saw_comma = false;
                let mut need_comma = false;
                while pairs.peek().is_some() {
                    if !self.comma_optional {
                        Self::syntax_error(
                            need_comma ^ saw_comma,
                            "expected comma",
                            npair.unwrap().as_span().end_pos(),
                        )?;
                    }
                    npair = pairs.peek();
                    let (node, comma) = self.handle_kvpair(&mut pairs)?;
                    kvs.push(node);
                    saw_comma = comma;
                    need_comma = true;
                }
                if npair.is_some() {
                    Self::syntax_error(
                        !self.comma_trailing && saw_comma,
                        "no comma expected",
                        npair.unwrap().as_span().end_pos(),
                    )?;
                }
                Ok(Document::Mapping(kvs))
            }
            Rule::array => {
                let mut pairs = pair.into_inner();
                let mut npair = pairs.peek();
                let mut values = Vec::new();
                let mut saw_comma = false;
                let mut need_comma = false;
                while pairs.peek().is_some() {
                    if !self.comma_optional {
                        Self::syntax_error(
                            need_comma ^ saw_comma,
                            "expected comma",
                            npair.unwrap().as_span().end_pos(),
                        )?;
                    }

                    npair = pairs.peek();
                    let (node, comma) = self.handle_array_elem(&mut pairs)?;
                    values.push(node);
                    saw_comma = comma;
                    need_comma = true;
                }
                if npair.is_some() {
                    Self::syntax_error(
                        !self.comma_trailing && saw_comma,
                        "no comma expected",
                        npair.unwrap().as_span().end_pos(),
                    )?;
                }

                Ok(Document::Sequence(values))
            }
            Rule::COMMENT => self.handle_comment(pair),
            Rule::EOI => Ok(Document::Null),
            Rule::text => {
                let mut doc = pair
                    .into_inner()
                    .map(|p| self.handle_pair(p))
                    .collect::<Result<Vec<_>, _>>()?;
                // Since we explicitly handled EOI, remove the dummy Null node
                // from the end of the vector.
                let _ = doc.pop();
                // A single node, or a sequence?
                if doc.len() == 1 {
                    Ok(doc.pop().unwrap())
                } else {
                    Ok(Document::Fragment(doc))
                }
            }

            _ => Err(Error::Unknown(format!("{:?}", pair))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result};

    #[test]
    fn test_null() -> Result<()> {
        let relax = Relax::default();
        let null = relax.from_str("null")?;
        assert!(matches!(null, Document::Null));
        Ok(())
    }

    #[test]
    fn test_boolean() -> Result<()> {
        let relax = Relax::default();
        let b = relax.from_str("true")?;
        assert!(matches!(b, Document::Boolean(true)));
        let b = relax.from_str("false")?;
        assert!(matches!(b, Document::Boolean(false)));
        Ok(())
    }

    fn parse_string(r: &Relax, text: &str) -> Result<String> {
        if let Document::String(s, _) = r.from_str(text)? {
            Ok(s)
        } else {
            Err(anyhow!("Didn't return Document::String()"))
        }
    }

    #[test]
    fn test_string() -> Result<()> {
        let relax = Relax::default();
        let s = parse_string(&relax, r#""foo""#)?;
        assert_eq!(s, "foo");
        let s = parse_string(&relax, r#" "\"\'\\\/\b\f\n\r\t\u2122\xac" "#)?;
        assert_eq!(s, "\"'\\/\u{8}\u{c}\n\r\t\u{2122}\u{00ac}");
        let s = parse_string(&relax, r#" "\e" "#);
        assert_eq!(s.unwrap_err().to_string(), "unhandled escape: `\\e`");
        let s = parse_string(&relax, r#" "\uD800" "#);
        assert_eq!(
            s.unwrap_err().to_string(),
            "converted integer out of range for `char`"
        );
        Ok(())
    }

    fn parse_integer(r: &Relax, text: &str) -> Result<i128> {
        if let Document::Int(int) = r.from_str(text)? {
            Ok(int.into())
        } else {
            Err(anyhow!("Didn't return Document::Int()"))
        }
    }

    fn parse_float(r: &Relax, text: &str) -> Result<f64> {
        if let Document::Float(f) = r.from_str(text)? {
            Ok(f)
        } else {
            Err(anyhow!("Didn't return Document::Float()"))
        }
    }

    #[test]
    fn test_number_bin() -> Result<()> {
        let relax = Relax::default();
        let i = parse_integer(&relax, "0b10100101")?;
        assert_eq!(i, 0xA5);
        let i = parse_integer(&relax, "-0b11111111")?;
        assert_eq!(i, -255);
        Ok(())
    }
    #[test]
    fn test_number_hex() -> Result<()> {
        let relax = Relax::default();
        let i = parse_integer(&relax, "0x1234")?;
        assert_eq!(i, 0x1234);
        let i = parse_integer(&relax, "-0x5678")?;
        assert_eq!(i, -0x5678);
        Ok(())
    }
    #[test]
    fn test_number_oct() -> Result<()> {
        let relax = Relax::default();
        let i = parse_integer(&relax, "0o755")?;
        assert_eq!(i, 0o755);
        let i = parse_integer(&relax, "-0o100")?;
        assert_eq!(i, -64);
        Ok(())
    }

    #[test]
    fn test_number_dec() -> Result<()> {
        let relax = Relax::default();
        let i = parse_integer(&relax, "+1234")?;
        assert_eq!(i, 1234);
        let i = parse_integer(&relax, "-5678")?;
        assert_eq!(i, -5678);
        Ok(())
    }

    #[test]
    fn test_number_float() -> Result<()> {
        let relax = Relax::default();
        let f = parse_float(&relax, "+1234.56")?;
        assert_eq!(f, 1234.56);
        let f = parse_float(&relax, "-5e6")?;
        assert_eq!(f, -5e6);
        let f = parse_float(&relax, "Infinity")?;
        assert_eq!(f, f64::INFINITY);
        Ok(())
    }

    fn parse_mapping(r: &Relax, text: &str) -> Result<Vec<Document>> {
        let doc = r.from_str(text)?;
        if let Document::Mapping(m) = doc {
            Ok(m)
        } else {
            Err(anyhow!("Didn't return Document::Mapping()\n{:?}", doc))
        }
    }

    fn kv_extract(kv: Option<&Document>) -> Result<(&str, &str)> {
        if let Some((Document::String(k, _), Document::String(v, _))) =
            kv.map(Document::as_kv).transpose()?
        {
            Ok((k.as_str(), v.as_str()))
        } else {
            Err(anyhow!("Expected KeyValue(String, String), not {:?}", kv))
        }
    }

    #[test]
    fn test_mapping() -> Result<()> {
        let relax = Relax::default();
        let mapping = parse_mapping(&relax, r#"{"foo": "bar", baz: "boo"}"#)?;
        let mut m = mapping.iter();
        let (k, v) = kv_extract(m.next())?;
        assert_eq!(k, "foo");
        assert_eq!(v, "bar");
        let (k, v) = kv_extract(m.next())?;
        assert_eq!(k, "baz");
        assert_eq!(v, "boo");
        assert!(m.next().is_none());
        Ok(())
    }

    fn parse_sequence(r: &Relax, text: &str) -> Result<Vec<Document>> {
        let doc = r.from_str(text);
        if let Ok(Document::Sequence(s)) = doc {
            Ok(s)
        } else {
            println!("doc = {:?}", doc);
            Err(anyhow!("Didn't return Document::Sequence()\n{:?}", doc))
        }
    }

    #[test]
    fn test_sequence() -> Result<()> {
        let relax = Relax::default();
        let sequence = parse_sequence(&relax, "[true, false, 3.14159]")?;
        let mut s = sequence.iter();
        assert!(matches!(s.next(), Some(Document::Boolean(true))));
        assert!(matches!(s.next(), Some(Document::Boolean(false))));
        assert!(matches!(s.next(), Some(Document::Float(_))));
        assert!(s.next().is_none());
        Ok(())
    }

    fn parse_comment(r: &Relax, text: &str) -> Result<(String, CommentFormat)> {
        let doc = r.from_str(text)?;
        if let Document::Comment(c, f) = doc {
            Ok((c, f))
        } else {
            Err(anyhow!("Didn't return Document::Comment()\n{:?}", doc))
        }
    }
    #[test]
    fn test_comment() -> Result<()> {
        let relax = Relax::default();
        let sequence = parse_sequence(
            &relax,
            r#"[
            // Some true value
            // extended
            // with more
            true,
            // A false value
            false,
            /*
             * Yet another value
             * but with a block
             * comment this time.
             */
            false
        ]"#,
        )?;
        match &sequence[..] {
            [Document::Fragment(a), Document::Fragment(b), Document::Fragment(c)] => {
                let mut i = a.iter();
                assert!(matches!(i.next(), Some(Document::Comment(_, _))));
                assert!(matches!(i.next(), Some(Document::Boolean(true))));
                assert!(i.next().is_none());
                let mut i = b.iter();
                assert!(matches!(i.next(), Some(Document::Comment(_, _))));
                assert!(matches!(i.next(), Some(Document::Boolean(false))));
                assert!(i.next().is_none());
                let mut i = c.iter();
                assert!(matches!(i.next(), Some(Document::Comment(_, _))));
                assert!(matches!(i.next(), Some(Document::Boolean(false))));
                assert!(i.next().is_none());
            }
            _ => return Err(anyhow!("Unexpected structure")),
        };

        let mapping = parse_mapping(
            &relax,
            r#"{
          // quoted
          "foo": "bar",
          baz: "boo" // bareword
        }"#,
        )?;
        match &mapping[..] {
            [Document::Fragment(a), Document::Fragment(b)] => {
                let mut i = a.iter();
                assert!(matches!(i.next(), Some(Document::Comment(_, _))));
                assert!(matches!(i.next(), Some(Document::String(_, _))));
                assert!(matches!(i.next(), Some(Document::String(_, _))));
                assert!(i.next().is_none());
                let mut i = b.iter();
                assert!(matches!(i.next(), Some(Document::String(_, _))));
                assert!(matches!(i.next(), Some(Document::String(_, _))));
                assert!(matches!(i.next(), Some(Document::Comment(_, _))));
                assert!(i.next().is_none());
            }
            _ => return Err(anyhow!("Unexpected structure")),
        };
        Ok(())
    }

    #[test]
    fn test_json_comment() -> Result<()> {
        let relax = Relax::json();
        assert!(parse_comment(&relax, "// foo").is_err());
        assert!(parse_comment(&relax, "# foo").is_err());
        assert!(parse_comment(&relax, "/* foo */").is_err());
        Ok(())
    }

    #[test]
    fn test_json5_comment() -> Result<()> {
        let relax = Relax::json5();
        assert!(parse_comment(&relax, "// foo").is_ok());
        assert!(parse_comment(&relax, "# foo").is_err());
        assert!(parse_comment(&relax, "/* foo */").is_ok());
        Ok(())
    }

    #[test]
    fn test_hjson_comment() -> Result<()> {
        let relax = Relax::hjson();
        assert!(parse_comment(&relax, "// foo").is_ok());
        assert!(parse_comment(&relax, "# foo").is_ok());
        assert!(parse_comment(&relax, "/* foo */").is_ok());
        Ok(())
    }

    #[test]
    fn test_json_commas() -> Result<()> {
        let relax = Relax::json();
        assert!(parse_sequence(&relax, "[true, false]").is_ok());
        assert!(parse_sequence(&relax, "[true, false,]").is_err());
        assert!(parse_sequence(&relax, "[true\nfalse]").is_err());

        assert!(parse_mapping(&relax, r#"{"a": true, "b": false}"#).is_ok());
        assert!(parse_mapping(&relax, r#"{"a": true, "b": false,}"#).is_err());
        assert!(parse_mapping(
            &relax,
            r#"{"a": true
                "b": false}"#
        )
        .is_err());
        assert!(parse_sequence(
            &relax,
            r#"[
            {k: 22,   m: 6,  code_type: "hsiao"  },
            {k: 22,   m: 5,  code_type: "hsiao"  },
        ]"#
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_json5_commas() -> Result<()> {
        let relax = Relax::json5();
        assert!(parse_sequence(&relax, "[true, false]").is_ok());
        assert!(parse_sequence(&relax, "[true, false,]").is_ok());
        assert!(parse_sequence(&relax, "[true\nfalse]").is_err());

        assert!(parse_mapping(&relax, r#"{"a": true, "b": false}"#).is_ok());
        assert!(parse_mapping(&relax, r#"{"a": true, "b": false,}"#).is_ok());
        assert!(parse_mapping(
            &relax,
            r#"{"a": true
                "b": false}"#
        )
        .is_err());
        assert!(parse_sequence(
            &relax,
            r#"[
            {k: 22,   m: 6,  code_type: "hsiao"  },
            {k: 22,   m: 5,  code_type: "hsiao"  },
        ]"#
        )
        .is_ok());
        Ok(())
    }

    #[test]
    fn test_hjson_commas() -> Result<()> {
        let relax = Relax::hjson();
        assert!(parse_sequence(&relax, "[true, false]").is_ok());
        assert!(parse_sequence(&relax, "[true, false,]").is_ok());
        assert!(parse_sequence(&relax, "[true\nfalse]").is_ok());

        assert!(parse_mapping(&relax, r#"{"a": true, "b": false}"#).is_ok());
        assert!(parse_mapping(&relax, r#"{"a": true, "b": false,}"#).is_ok());
        assert!(parse_mapping(
            &relax,
            r#"{"a": true
                "b": false}"#
        )
        .is_ok());
        assert!(parse_sequence(
            &relax,
            r#"[
            {k: 22,   m: 6,  code_type: "hsiao"  }
            {k: 22,   m: 5,  code_type: "hsiao"  }
        ]"#
        )
        .is_ok());
        Ok(())
    }

    #[test]
    fn test_json_simple_string() -> Result<()> {
        let relax = Relax::json();
        assert!(parse_string(&relax, r#""foo""#).is_ok());
        assert!(parse_string(&relax, r#"'foo'"#).is_err());
        assert!(parse_string(&relax, "foo bar baz").is_err());
        assert!(parse_mapping(&relax, "{a: true}").is_err());
        let m = parse_mapping(
            &relax,
            r#"
            {
                time: 01/02/03 04:05:06AM
                name: Fred
                type: person
            }
        "#,
        );
        assert!(m.is_err());
        assert!(parse_string(
            &relax,
            r#""foo\
            bar""#
        )
        .is_err());
        assert!(parse_string(
            &relax,
            r#"
            '''
            "Weird Al"
            Yankovic!
            '''"#
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_json5_string() -> Result<()> {
        let relax = Relax::json5();
        assert!(parse_string(&relax, r#""foo""#).is_ok());
        assert!(parse_string(&relax, r#"'foo'"#).is_ok());
        assert!(parse_string(&relax, "foo bar baz").is_err());
        assert!(parse_mapping(&relax, "{a: true}").is_ok());
        let m = parse_mapping(
            &relax,
            r#"
            {
                time: 01/02/03 04:05:06AM
                name: Fred
                type: person
            }
        "#,
        );
        assert!(m.is_err());
        assert!(parse_string(
            &relax,
            r#""foo\
            bar""#
        )
        .is_ok());
        assert!(parse_string(
            &relax,
            r#"
            '''
            "Weird Al"
            Yankovic!
            '''"#
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn test_hjson_string() -> Result<()> {
        let relax = Relax::hjson();
        assert!(parse_string(&relax, r#""foo""#).is_ok());
        assert!(parse_string(&relax, r#"'foo'"#).is_ok());
        assert!(parse_string(&relax, "foo bar baz\n").is_ok());
        assert!(parse_mapping(&relax, "{a: true}").is_ok());
        let m = parse_mapping(
            &relax,
            r#"
            {
                time: 01/02/03 04:05:06AM
                name: Fred
                type: person
            }
        "#,
        );
        assert!(m.is_ok());
        assert!(parse_string(
            &relax,
            r#""foo\
            bar""#
        )
        .is_err());
        assert!(parse_string(
            &relax,
            r#"
            '''
            "Weird Al"
            Yankovic!
            '''"#
        )
        .is_ok());
        Ok(())
    }
}
