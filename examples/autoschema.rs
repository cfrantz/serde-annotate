/// Work-in-Progress.  This program is not done yet.
///
/// The `autoschema` program scans over a corpus of input documents and emits
/// a description of the kinds of nodes seen at each DocPath in the document.
/// This information can then be used to auto-generate a schema for the input
/// documents.
///
/// IOW, this program helps you do what you should have done when you thought
/// "Who cares? Its just JSON! It's schema free!".
use anstyle::{AnsiColor, Style};
use anyhow::Result;
use clap::Parser;
use serde_annotate::{DocPath, Document, Int};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
struct ColorProfile {
    error: Style,
    ok: Style,
}

impl ColorProfile {
    pub fn basic() -> Self {
        ColorProfile {
            error: AnsiColor::Red.on_default(),
            ok: AnsiColor::Green.on_default(),
        }
    }
}

#[derive(Debug, Default)]
struct Schema {
    null: u32,
    boolean: u32,
    string: u32,
    integer: u32,
    float: u32,
    object: u32,
    array: u32,
    total: u32,
    children: HashMap<String, Schema>,
}

impl Schema {
    fn get_mut(&mut self, path: &[DocPath]) -> &mut Self {
        let mut ret = self;
        let n = path.len() - 1;
        for (i, p) in path.iter().enumerate() {
            let (key, obj, arr) = match p {
                DocPath::Name(x) => (*x, 1, 0),
                DocPath::Index(_) => ("[_]", 0, 1),
            };
            if i <= n {
                ret.total += 1;
                ret.object += obj;
                ret.array += arr;
            }
            ret = ret.children.entry(key.to_string()).or_default();
        }
        ret
    }

    fn check_str(&mut self, s: &str) {
        if let Ok(_) = Int::from_str_radix(s, 0) {
            self.integer += 1;
        } else {
            match s {
                "true" | "True" | "TRUE" => self.boolean += 1,
                "false" | "False" | "FALSE" => self.boolean += 1,
                _ => self.string += 1,
            }
        }
    }

    fn detect(&mut self, root: &Document) {
        for (path, doc) in root.iter_path() {
            let node = self.get_mut(&path);
            node.total += 1;
            match doc {
                Document::Null => node.null += 1,
                Document::Boolean(_) => node.boolean += 1,
                Document::String(s, _) => node.check_str(s.as_str()),
                Document::StaticStr(s, _) => node.check_str(s),
                Document::Int(_) => node.integer += 1,
                Document::Float(_) => node.float += 1,
                _ => {
                    panic!("Unexpected node {:?}", node);
                }
            }
        }
    }

    fn print(&self, name: &str, indent: usize, color: &ColorProfile) {
        let good = self.total == self.null
            || self.total == self.boolean
            || self.total == self.string
            || self.total == self.integer
            || self.total == self.float
            || self.total == self.object
            || self.total == self.array;

        print!("{}", (if good { color.ok } else { color.error }).render());
        print!("{0:>1$}|{2:->20}: ", "", indent * 4, name,);
        print!(
            "(n:{:<3} b:{:<3} s:{:<3} i:{:<3} f:{:<3} o:{:<3} a:{:<3}) / {:<3}",
            self.null,
            self.boolean,
            self.string,
            self.integer,
            self.float,
            self.object,
            self.array,
            self.total,
        );
        println!("{}", color.ok.render_reset());
        for (k, v) in self.children.iter() {
            v.print(k.as_str(), indent + 1, color)
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(name = "FILES")]
    files: Vec<PathBuf>,

    #[clap(short, long, value_parser)]
    color: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let color = if args.color {
        ColorProfile::basic()
    } else {
        ColorProfile::default()
    };

    let mut schema = Schema::default();
    for f in args.files {
        println!("Checking {:?}", f);
        let text = std::fs::read_to_string(f)?;
        let document = Document::parse(&text)?;
        schema.detect(&document);
    }
    schema.print("", 0, &color);
    Ok(())
}
