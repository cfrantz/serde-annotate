use anyhow::Result;
use serde_annotate::annotate::Annotate;
use serde_annotate::serialize;
use serde_derive::{Deserialize, Serialize};

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
    tester!(
        json,
        Coordinate,
        &value,
        r#"
        {
          "x": 16,
          "y": 10,
          "z": 8
        }"#
    );

    tester!(
        json5,
        Coordinate,
        &value,
        r#"
        {
          // X-coordinate
          x: 0x10,
          // Y-coordinate
          y: 10,
          // Z-coordinate
          z: 8
        }"#
    );

    tester!(
        hjson,
        Coordinate,
        &value,
        r#"
        {
          # X-coordinate
          x: 16,
          # Y-coordinate
          y: 10,
          # Z-coordinate
          z: 8
        }"#
    );

    tester!(
        yaml,
        Coordinate,
        &value,
        r#"
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

// A container struct which does not implement Annotate.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Sfdp {
    header: SfdpHeader,
}

// Numbers in different bases, comments from functions within the impl.
#[derive(Serialize, Deserialize, Annotate, Debug, PartialEq)]
struct SfdpHeader {
    #[annotate(format=hex, comment=_signature())]
    signature: u32,
    #[annotate(comment = "SFDP Version")]
    minor: u8,
    major: u8,
    #[annotate(comment = "Number of parameter headers (minus 1)")]
    nph: u8,
    #[annotate(format=bin, comment="Reserved field should be all ones")]
    reserved: u8,
}

impl SfdpHeader {
    fn _signature(&self) -> Option<String> {
        Some(format!(
            "Signature value='{}' (should be 'SFDP')",
            self.signature
                .to_le_bytes()
                .map(|b| char::from(b).to_string())
                .join("")
        ))
    }
}

#[test]
fn test_sfdp() -> Result<()> {
    let value = Sfdp {
        header: SfdpHeader {
            signature: 0x50444653,
            minor: 6,
            major: 1,
            nph: 2,
            reserved: 255,
        },
    };

    tester!(
        json,
        Sfdp,
        &value,
        r#"
        {
          "header": {
            "signature": 1346651731,
            "minor": 6,
            "major": 1,
            "nph": 2,
            "reserved": 255
          }
        }"#
    );

    tester!(
        json5,
        Sfdp,
        &value,
        r#"
        {
          header: {
            // Signature value='SFDP' (should be 'SFDP')
            signature: 0x50444653,
            // SFDP Version
            minor: 6,
            major: 1,
            // Number of parameter headers (minus 1)
            nph: 2,
            // Reserved field should be all ones
            reserved: 255
          }
        }"#
    );

    tester!(
        yaml,
        Sfdp,
        &value,
        r#"
        ---
        header:
          # Signature value='SFDP' (should be 'SFDP')
          signature: 0x50444653
          # SFDP Version
          minor: 6
          major: 1
          # Number of parameter headers (minus 1)
          nph: 2
          # Reserved field should be all ones
          reserved: 0b11111111"#
    );

    Ok(())
}

#[derive(Serialize, Deserialize, Annotate, Debug, PartialEq)]
enum NesAddress {
    #[annotate(format=compact, comment="NES file offset")]
    File(u32),
    #[annotate(format=compact, comment="NES PRG bank:address")]
    Prg(#[annotate(format=hex)] u8, #[annotate(format=hex)] u16),
    #[annotate(format=compact, comment="NES CHR bank:address")]
    Chr(#[annotate(format=hex)] u8, #[annotate(format=hex)] u16),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Addresses {
    a: NesAddress,
    b: NesAddress,
    c: NesAddress,
}

#[test]
fn test_nes_addresses() -> Result<()> {
    let value = Addresses {
        a: NesAddress::File(0x4010),
        b: NesAddress::Prg(1, 0x8000),
        c: NesAddress::Chr(2, 0x400),
    };

    tester!(
        json,
        Addresses,
        &value,
        r#"
        {
          "a": {
            "File": 16400
          },
          "b": {
            "Prg": [1, 32768]
          },
          "c": {
            "Chr": [2, 1024]
          }
        }"#
    );

    tester!(
        json5,
        Addresses,
        &value,
        r#"
        {
          a: {
            // NES file offset
            File: 16400
          },
          b: {
            // NES PRG bank:address
            Prg: [0x1, 0x8000]
          },
          c: {
            // NES CHR bank:address
            Chr: [0x2, 0x400]
          }
        }"#
    );

    tester!(
        yaml,
        Addresses,
        &value,
        r#"
          ---
          a:
            # NES file offset
            File: 16400
          b:
            # NES PRG bank:address
            Prg: [0x1, 0x8000]
          c:
            # NES CHR bank:address
            Chr: [0x2, 0x400]"#
    );

    Ok(())
}
