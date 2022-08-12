use anyhow::{anyhow, Result};
use clap::{ArgEnum, Parser};
use serde_annotate::{ColorProfile, Document};
use std::path::PathBuf;

#[derive(ArgEnum, Clone, Copy, Debug)]
enum Format {
    Json,
    Json5,
    Hjson,
    Yaml,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, arg_enum, value_parser)]
    format: Format,

    #[clap(short, long, value_parser)]
    color: bool,

    #[clap(name = "FILE")]
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let text = std::fs::read_to_string(&args.file)?;
    let document = Document::parse(&text)?;

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
