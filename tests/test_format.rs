use anyhow::Result;
use serde_annotate::annotate::Annotate;
use serde_annotate::serialize;
use serde_derive::{Deserialize, Serialize};
use std::io::Cursor;

fn fixdoc(doc: &str) -> String {
    let mut s = String::new();
    let mut dedent: Option<usize> = None;
    for line in doc.split('\n') {
        if line.is_empty() && dedent.is_none() {
            continue;
        }
        if dedent.is_none() {
            dedent = line.find(|c: char| !c.is_whitespace());
            assert!(dedent.is_some());
        }
        let (_, v) = line.split_at(dedent.unwrap());
        s.push_str(v);
        s.push('\n');
    }
    s.pop();
    s
}

// Emit and check the string form out of our serializer.
// Parse through the published deserializers and check for equality.
macro_rules! tester {
    (json, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_json().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = serde_json::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
    (json5, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_json5().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = json5::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
    (hjson, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_hjson().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = deser_hjson::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
    (yaml, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_yaml().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = serde_yaml::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
}

#[derive(Serialize, Deserialize, Annotate, Debug, PartialEq)]
struct Coordinate {
    #[annotate(format=hex, comment="X-coordinate")]
    pub x: u32,
    #[annotate(format=dec, comment="Y-coordinate")]
    pub y: u32,
    #[annotate(format=oct, comment="Z-coordinate")]
    pub z: u32,
}

#[test]
fn test_coordinate() -> Result<()> {
    let value = Coordinate { x: 16, y: 10, z: 8 };
    tester!(json, Coordinate, &value, r#"
        {
          "x": 16,
          "y": 10,
          "z": 8
        }"#
    );

    tester!(json5, Coordinate, &value, r#"
        {
          // X-coordinate
          x: 0x10,
          // Y-coordinate
          y: 10,
          // Z-coordinate
          z: 8
        }"#
    );

    tester!(hjson, Coordinate, &value, r#"
        {
          # X-coordinate
          x: 16,
          # Y-coordinate
          y: 10,
          # Z-coordinate
          z: 8
        }"#
    );

    tester!(yaml, Coordinate, &value, r#"
        ---
        # X-coordinate
        x: 0x10
        # Y-coordinate
        y: 10
        # Z-coordinate
        z: 0o10"#
    );

    Ok(())
}
