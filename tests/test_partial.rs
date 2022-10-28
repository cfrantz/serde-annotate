#![feature(min_specialization)]
use anyhow::Result;
use serde_annotate::serialize;
use serde_annotate::{Document, StrFormat};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Partial {
    n: i32,
    doc: Document,
}

const SERIALIZE_RESULT: &str = r#"{
  n: 5,
  doc: [
    "Hello",
    "world"
  ]
}"#;

#[test]
fn test_partial_serialize() -> Result<()> {
    let p = Partial {
        n: 5,
        doc: Document::Sequence(vec![
            Document::String("Hello".into(), StrFormat::Standard),
            Document::String("world".into(), StrFormat::Standard),
        ]),
    };
    let s = serialize(&p)?.to_json5().to_string();
    assert_eq!(s, SERIALIZE_RESULT);
    Ok(())
}

#[test]
fn test_partial_serialize_error() -> Result<()> {
    let p = Partial {
        n: 5,
        doc: Document::Sequence(vec![
            Document::String("Hello".into(), StrFormat::Standard),
            Document::String("world".into(), StrFormat::Standard),
        ]),
    };
    let s = serde_json::to_string_pretty(&p);
    assert!(s.is_err());
    assert_eq!(
        s.unwrap_err().to_string(),
        "Serializing document nodes is only supported with serde_annotate::AnnotatedSerializer"
    );
    Ok(())
}

#[test]
fn test_partial_deserialize() -> Result<()> {
    let doc = r#"{
        n: 10,
        doc: {
            # A comment
            key: "value",
            i: 5,
            j: 10,
        }
    }"#;
    let p = serde_annotate::from_str::<Partial>(doc)?;
    assert_eq!(p.n, 10);
    let Document::Mapping(m) = p.doc else {
        panic!("Expecting Document::Mapping");
    };
    let (k, v) = m[0].as_kv()?;
    assert_eq!(k.as_str()?, "key");
    assert_eq!(v.as_str()?, "value");
    let (k, v) = m[1].as_kv()?;
    assert_eq!(k.as_str()?, "i");
    assert_eq!(u32::try_from(v)?, 5);
    let (k, v) = m[2].as_kv()?;
    assert_eq!(k.as_str()?, "j");
    assert_eq!(u32::try_from(v)?, 10);
    Ok(())
}

#[test]
fn test_partial_deserialize_error() -> Result<()> {
    let p = serde_json::from_str::<Partial>(r#"{"n":5, "doc": []}"#);
    assert!(p.is_err());
    assert_eq!(p.unwrap_err().to_string(),
        "Deserializing document nodes is only supported with serde_annotate::Deserializer at line 1 column 15");
    Ok(())
}
