use pest::error::Error as PestError;
use pest::iterators::{Pair, Pairs};
use pest::Parser as P;
use pest::Position;
use pest_derive::Parser;

use crate::document::{CommentFormat, Document, StrFormat};
use crate::error::Error;
use crate::integer::{Base, Int};

#[derive(Parser)]
#[grammar = "relax.pest"]
pub struct Relax {
    pub comma_trailing: bool,
    pub comma_optional: bool,
    pub number_hex: bool,
    pub number_plus: bool,
    pub number_leading_dec: bool,
    pub number_trailing_dec: bool,
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
    /// The default Relax parser is maximally permissive.
    fn default() -> Self {
        Relax {
            comma_trailing: true,
            comma_optional: true,
            number_hex: true,
            number_plus: true,
            number_leading_dec: true,
            number_trailing_dec: true,
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
        Relax {
            comma_trailing: false,
            comma_optional: false,
            number_hex: false,
            number_plus: false,
            number_leading_dec: false,
            number_trailing_dec: false,
            string_single_quote: false,
            string_unquoted: false,
            string_ident: false,
            string_json5_multiline: false,
            string_hjson_multiline: false,
            comment_slash: false,
            comment_hash: false,
            comment_block: false,
        }
    }

    pub fn json5() -> Self {
        let mut r = Self::default();
        r.comma_optional = false;
        r.string_unquoted = false;
        r.string_hjson_multiline = false;
        r.comment_hash = false;
        r
    }

    pub fn hjson() -> Self {
        let mut r = Self::default();
        r.string_json5_multiline = false;
        r.number_hex = false;
        r.number_plus = false;
        r.number_leading_dec = false;
        r.number_trailing_dec = false;
        r
    }

    pub fn from_str(&self, text: &str) -> Result<Document, Error> {
        let json = Relax::parse(Rule::text, text)?.next().unwrap();
        self.handle_pair(json)
    }

    fn handle_number(&self, text: &str) -> Result<Document, Error> {
        let mut negative = false;
        let t = if let Some(t) = text.strip_prefix('+') {
            t
        } else if let Some(t) = text.strip_prefix('-') {
            negative = true;
            t
        } else {
            text
        };
        if t.starts_with("0x") || t.starts_with("0X") {
            // Hexadecimal integer.
            let (_, t) = t.split_at(2);
            let val = u128::from_str_radix(t, 16).unwrap();
            if negative {
                let val = -(val as i128);
                return Ok(Document::Int(Int::new(val, Base::Hex)));
            } else {
                return Ok(Document::Int(Int::new(val, Base::Hex)));
            }
        } else if t.contains('.')
            || t.contains('e')
            || t.contains('E')
            || t == "NaN"
            || t == "Infinity"
        {
            // Floating point number.
            return Ok(Document::Float(text.parse().unwrap()));
        } else {
            // Decimal integer.
            let val = u128::from_str_radix(t, 10).unwrap();
            if negative {
                let val = -(val as i128);
                return Ok(Document::Int(Int::new(val, Base::Dec)));
            } else {
                return Ok(Document::Int(Int::new(val, Base::Dec)));
            }
        }
    }

    fn handle_kvpair(&self, pairs: &mut Pairs<Rule>) -> Result<Document, Error> {
        let mut k = usize::MAX;
        let mut v = usize::MAX;
        let mut kv = vec![];
        loop {
            let pair = pairs.peek();
            if pair.is_none() {
                break;
            }
            let pair = pair.unwrap();
            let (ln, _) = pair.as_span().start_pos().line_col();
            let node = self.handle_pair(pair)?;
            let is_comment = node.comment().is_some();
            if k == usize::MAX || v == usize::MAX {
                if is_comment {
                    // Keep
                } else if k == usize::MAX {
                    k = ln;
                } else {
                    v = ln;
                }
            } else if is_comment && v == ln {
                // Keep
            } else {
                // Don't keep - we're done with this kvpair.
                break;
            }
            kv.push(node);
            // Advance the iterator.
            let _ = pairs.next();
        }
        Ok(Document::Fragment(kv))
    }

    fn handle_array_elem(&self, pairs: &mut Pairs<Rule>) -> Result<(Document, bool), Error> {
        let mut i = usize::MAX;
        let mut item = vec![];
        let mut comma = false;
        let mut saw_value = false;
        loop {
            let pair = pairs.peek();
            if pair.is_none() {
                break;
            }
            let pair = pair.unwrap();
            let rule = pair.as_rule();
            if rule == Rule::comma {
                let _ = pairs.next();
                comma = true;
                continue;
            }
            let (line, _) = pair.as_span().start_pos().line_col();
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
                    // Comment is before the vale, keep the comment.
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
            let lines = c.split("\n").map(str::trim).collect::<Vec<_>>();
            let lines = Self::strip_leading_prefix(&lines, '*');
            let lines = Self::strip_leading_prefix(&lines, ' ');
            let start = if lines.get(0).map(|s| s.is_empty()) == Some(true) {
                1
            } else {
                0
            };
            let c = lines[start..].join("\n");
            Ok(Document::Comment(c, CommentFormat::Normal))
        } else if comment.starts_with("//") {
            Self::syntax_error(
                !self.comment_slash,
                "slash comment",
                pair.as_span().start_pos(),
            )?;
            let lines = comment.split("\n").map(str::trim).collect::<Vec<_>>();
            let lines = Self::strip_leading_prefix(&lines, '/');
            let lines = Self::strip_leading_prefix(&lines, ' ');
            let end = lines.len()
                - if lines.last().map(|s| s.is_empty()) == Some(true) {
                    1
                } else {
                    0
                };
            let c = lines[..end].join("\n");
            Ok(Document::Comment(c, CommentFormat::Normal))
        } else if comment.starts_with("#") {
            Self::syntax_error(
                !self.comment_hash,
                "hash comment",
                pair.as_span().start_pos(),
            )?;
            let lines = comment.split("\n").map(str::trim).collect::<Vec<_>>();
            let lines = Self::strip_leading_prefix(&lines, '#');
            let lines = Self::strip_leading_prefix(&lines, ' ');
            let end = lines.len()
                - if lines.last().map(|s| s.is_empty()) == Some(true) {
                    1
                } else {
                    0
                };
            let c = lines[..end].join("\n");
            Ok(Document::Comment(c, CommentFormat::Normal))
        } else {
            Err(Error::Unknown(comment.into()))
        }
    }

    fn handle_pair(&self, pair: Pair<Rule>) -> Result<Document, Error> {
        match pair.as_rule() {
            //Rule::value => self.handle_pair(pair.into_inner().next().unwrap()),
            Rule::null => Ok(Document::Null),
            Rule::boolean => Ok(Document::Boolean(pair.as_str().parse().unwrap())),
            Rule::string => {
                let s = pair.as_str();
                let end = s.len() - 1;
                Ok(Document::String(s[1..end].into(), StrFormat::Standard))
            }
            Rule::identifier => {
                // TODO: add StrFormat::Unquoted
                Ok(Document::String(pair.as_str().into(), StrFormat::Standard))
            }
            Rule::number => self.handle_number(pair.as_str()),
            Rule::object => {
                let mut pairs = pair.into_inner();
                let mut kvs = Vec::new();
                while pairs.peek().is_some() {
                    kvs.push(self.handle_kvpair(&mut pairs)?);
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
                Self::syntax_error(
                    !self.comma_trailing && saw_comma,
                    "no comma expected",
                    npair.unwrap().as_span().end_pos(),
                )?;
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

            _ => Err(Error::Unknown(format!("{:?}", pair)).into()),
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
    fn test_number_hex() -> Result<()> {
        let relax = Relax::default();
        let i = parse_integer(&relax, "0x1234")?;
        assert_eq!(i, 0x1234);
        let i = parse_integer(&relax, "-0x5678")?;
        assert_eq!(i, -0x5678);
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
        if let Document::Mapping(m) = r.from_str(text)? {
            Ok(m)
        } else {
            Err(anyhow!("Didn't return Document::Mapping()"))
        }
    }

    fn kv_extract<'a>(kv: Option<&'a Document>) -> Result<(&'a str, &'a str)> {
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
        if let Document::Sequence(s) = r.from_str(text)? {
            Ok(s)
        } else {
            Err(anyhow!("Didn't return Document::Mapping()"))
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
        if let Document::Comment(c, f) = r.from_str(text)? {
            Ok((c, f))
        } else {
            Err(anyhow!("Didn't return Document::Comment()"))
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
        let k = parse_sequence(&relax, "[true\nfalse]");
        println!("k={:?}", k);
        assert!(parse_sequence(&relax, "[true\nfalse]").is_err());
        Ok(())
    }

    #[test]
    fn test_json5_commas() -> Result<()> {
        let relax = Relax::json5();
        assert!(parse_sequence(&relax, "[true, false]").is_ok());
        assert!(parse_sequence(&relax, "[true, false,]").is_ok());
        assert!(parse_sequence(&relax, "[true\nfalse]").is_err());
        Ok(())
    }

    #[test]
    fn test_hjson_commas() -> Result<()> {
        let relax = Relax::hjson();
        assert!(parse_sequence(&relax, "[true, false]").is_ok());
        assert!(parse_sequence(&relax, "[true, false,]").is_ok());
        assert!(parse_sequence(&relax, "[true\nfalse]").is_ok());
        Ok(())
    }
}
