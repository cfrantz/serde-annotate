#![feature(min_specialization)]
use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_annotate::{serialize, Annotate, ColorProfile};

#[derive(Serialize, Deserialize, Annotate, Debug, PartialEq)]
struct Coordinate {
    #[annotate(format=hex, comment="X-coordinate")]
    pub x: u32,
    #[annotate(format=dec, comment="Y-coordinate")]
    pub y: u32,
    #[annotate(format=oct, comment="Z-coordinate")]
    pub z: u32,
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

#[derive(Serialize, Deserialize, Annotate, Debug, PartialEq)]
struct Everything {
    #[annotate(comment = "Basic String")]
    string: String,
    #[annotate(format=block, comment="Multiline String")]
    multiline: String,
    #[serde(with = "serde_bytes")]
    #[annotate(format=hex, comment="Bytes")]
    bytes: Vec<u8>,
    #[annotate(comment = "Integer")]
    num: u32,
    #[annotate(format=hex, comment="Integer (hex)")]
    hex: u32,
    #[annotate(format=oct, comment="Integer (octal)")]
    oct: u32,
    #[annotate(format=bin, comment="Integer (binary)")]
    bin: u32,
    #[annotate(comment = "Boolean")]
    boolean: bool,
    #[annotate(comment = "A null value")]
    no_value: Option<String>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Format {
    Json,
    Json5,
    Hjson,
    Yaml,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, value_parser)]
    structure: String,

    #[clap(short, long, value_enum, value_parser)]
    format: Format,

    #[clap(short, long, value_parser)]
    color: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let document = match args.structure.as_str() {
        "coordinate" => {
            let value = Coordinate { x: 16, y: 10, z: 8 };
            serialize(&value)?
        }
        "sfdp" => {
            let value = Sfdp {
                header: SfdpHeader {
                    signature: 0x50444653,
                    minor: 6,
                    major: 1,
                    nph: 2,
                    reserved: 255,
                },
            };
            serialize(&value)?
        }
        "nes" => {
            let value = Addresses {
                a: NesAddress::File(0x4010),
                b: NesAddress::Prg(1, 0x8000),
                c: NesAddress::Chr(2, 0x400),
            };
            serialize(&value)?
        }
        "everything" => {
            let value = Everything {
                string: "This is a string\nwith some\nescapes".into(),
                multiline: "This is\na multiline\nstring!".into(),
                bytes: vec![
                    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x10, 0x11, 0x12, 0x13, 0x14,
                    0x15, 0x16, 0x17, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x30, 0x31,
                    0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                ],
                num: 100,
                hex: 0xdecaf,
                oct: 0o755,
                bin: 0b10010110,
                boolean: true,
                no_value: None,
            };
            serialize(&value)?
        }

        _ => {
            return Err(anyhow!("Unknown structure: {}", args.structure));
        }
    };

    let profile = ColorProfile::basic();
    let s = match args.format {
        Format::Json => {
            let mut d = document.to_json();
            if args.color {
                d = d.color(profile);
            }
            d.to_string()
        }
        Format::Json5 => {
            let mut d = document.to_json5();
            if args.color {
                d = d.color(profile);
            }
            d.to_string()
        }
        Format::Hjson => {
            let mut d = document.to_hjson();
            if args.color {
                d = d.color(profile);
            }
            d.to_string()
        }
        Format::Yaml => {
            let mut d = document.to_yaml();
            if args.color {
                d = d.color(profile);
            }
            d.to_string()
        }
    };

    println!("{}", s);
    Ok(())
}
