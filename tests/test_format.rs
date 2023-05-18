#![feature(min_specialization)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_annotate::serialize;
use serde_annotate::Annotate;

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
    (relax, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_json().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = serde_annotate::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
    (relax_json5, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_json5().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = serde_annotate::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
    (relax_hjson, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_hjson().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = serde_annotate::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
    (yaml, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_yaml().to_string();
        assert_eq!(string, fixdoc($expect));
        let decode: $t = serde_yaml::from_str(&string)?;
        assert_eq!($value, &decode);
    }};
    (ser_yaml, $t:ty, $value:expr, $expect:expr) => {{
        let doc = serialize($value)?;
        let string = doc.to_yaml().to_string();
        assert_eq!(string, fixdoc($expect));
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
    // TODO(serde-annotate#6): Currently, we do not emit comments for unit variants.
    #[annotate(comment = "Bad Address")]
    Invalid,
}

// TODO(serde-annotate#6): Currently, we do not emit comments for newtype structs.
#[derive(Serialize, Deserialize, Annotate, Debug, PartialEq)]
struct CpuAddress(#[annotate(format=hex, comment="CPU Address")] u16);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Addresses {
    a: NesAddress,
    b: NesAddress,
    c: Option<NesAddress>,
    // Note: using a vec here causes the serializer to request
    // serializing &CpuAddress rather than CpuAddress.  This tests
    // that the specializations of Annotate for &T are working.
    vectors: Vec<CpuAddress>,
    inv: NesAddress,
}

#[test]
fn test_nes_addresses() -> Result<()> {
    let value = Addresses {
        a: NesAddress::File(0x4010),
        b: NesAddress::Prg(1, 0x8000),
        c: Some(NesAddress::Chr(2, 0x400)),
        vectors: vec![CpuAddress(0xFFFA), CpuAddress(0xFFFC), CpuAddress(0xFFFE)],
        inv: NesAddress::Invalid,
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
          },
          "vectors": [
            65530,
            65532,
            65534
          ],
          "inv": "Invalid"
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
          },
          vectors: [
            0xFFFA,
            0xFFFC,
            0xFFFE
          ],
          inv: "Invalid"
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
            Chr: [0x2, 0x400]
          vectors:
            - 0xFFFA
            - 0xFFFC
            - 0xFFFE
          inv: Invalid"#
    );

    Ok(())
}

#[derive(Serialize, Deserialize, Annotate, Debug, PartialEq)]
struct Poem {
    #[serde(with = "serde_bytes")]
    #[annotate(comment = "No special bytes encoding")]
    first_stanza: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[annotate(format=hexstr, comment="Encoded as `hexstr`")]
    second_stanza: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[annotate(format=hexdump, comment="Encoded as `hexdump`")]
    third_stanza: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[annotate(format=xxd, comment="Encoded as `xxd`")]
    fourth_stanza: Vec<u8>,
}

#[test]
fn test_bytes() -> Result<()> {
    let value = Poem {
        first_stanza: "Mary had a little lamb".into(),
        second_stanza: "its fleece was white as snow".into(),
        third_stanza: "Everywhere that Mary went".into(),
        fourth_stanza: "the lamb was sure to go".into(),
    };

    // We can't test deserialization of the yaml data yet because we don't yet
    // have a parser or adapter for yaml that emits a `serde_annotate::Document`
    // to feed to our deserializer implementation.
    tester!(
        ser_yaml,
        Poem,
        &value,
        r#"
          ---
          # No special bytes encoding
          first_stanza: [
          0x4D,0x61,0x72,0x79,0x20,0x68,0x61,0x64,0x20,0x61,0x20,0x6C,0x69,0x74,0x74,0x6C,
          0x65,0x20,0x6C,0x61,0x6D,0x62,
          ]
          # Encoded as `hexstr`
          second_stanza: 69747320666c656563652077617320776869746520617320736e6f77
          # Encoded as `hexdump`
          third_stanza: |-
            00000000  45 76 65 72 79 77 68 65  72 65 20 74 68 61 74 20  |Everywhere that |
            00000010  4d 61 72 79 20 77 65 6e  74                       |Mary went|
          # Encoded as `xxd`
          fourth_stanza: |-
            00000000: 7468 6520 6c61 6d62 2077 6173 2073 7572  the lamb was sur
            00000010: 6520 746f 2067 6f                        e to go"#
    );

    // We can only test deserialization of the data with the `relax` parser
    // as it feeds our deserializer implementation.
    tester!(
        relax_json5,
        Poem,
        &value,
        r#"
        {
          // No special bytes encoding
          first_stanza: [
            77,
            97,
            114,
            121,
            32,
            104,
            97,
            100,
            32,
            97,
            32,
            108,
            105,
            116,
            116,
            108,
            101,
            32,
            108,
            97,
            109,
            98
          ],
          // Encoded as `hexstr`
          second_stanza: "69747320666c656563652077617320776869746520617320736e6f77",
          // Encoded as `hexdump`
          third_stanza: "00000000  45 76 65 72 79 77 68 65  72 65 20 74 68 61 74 20  |Everywhere that |\
        00000010  4d 61 72 79 20 77 65 6e  74                       |Mary went|",
          // Encoded as `xxd`
          fourth_stanza: "00000000: 7468 6520 6c61 6d62 2077 6173 2073 7572  the lamb was sur\
        00000010: 6520 746f 2067 6f                        e to go"
        }"#
    );

    // We can only test deserialization of the data with the `relax` parser
    // as it feeds our deserializer implementation.
    tester!(
        relax_hjson,
        Poem,
        &value,
        r#"
        {
          # No special bytes encoding
          first_stanza: [
            77,
            97,
            114,
            121,
            32,
            104,
            97,
            100,
            32,
            97,
            32,
            108,
            105,
            116,
            116,
            108,
            101,
            32,
            108,
            97,
            109,
            98
          ],
          # Encoded as `hexstr`
          second_stanza: "69747320666c656563652077617320776869746520617320736e6f77",
          # Encoded as `hexdump`
          third_stanza: 
            '''
            00000000  45 76 65 72 79 77 68 65  72 65 20 74 68 61 74 20  |Everywhere that |
            00000010  4d 61 72 79 20 77 65 6e  74                       |Mary went|
            ''',
          # Encoded as `xxd`
          fourth_stanza: 
            '''
            00000000: 7468 6520 6c61 6d62 2077 6173 2073 7572  the lamb was sur
            00000010: 6520 746f 2067 6f                        e to go
            '''
        }"#
    );

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Data {
    None,
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<Data>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct VariousData {
    none: Data,
    boolean: Data,
    integer: Data,
    string: Data,
    list: Data,
}

#[test]
fn test_untagged_data() -> Result<()> {
    let value = VariousData {
        none: Data::None,
        boolean: Data::Bool(true),
        integer: Data::Int(100),
        string: Data::Str("hello".into()),
        list: Data::List(vec![
            Data::None,
            Data::Bool(false),
            Data::Int(200),
            Data::Str("bye".into()),
        ]),
    };

    tester!(
        relax, // Use the 'relax' parser to test our deserializer.
        VariousData,
        &value,
        r#"
        {
          "none": null,
          "boolean": true,
          "integer": 100,
          "string": "hello",
          "list": [
            null,
            false,
            200,
            "bye"
          ]
        }"#
    );
    Ok(())
}
