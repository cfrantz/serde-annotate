#![feature(min_specialization)]
pub mod annotate;
mod color;
mod de;
mod doc_iter;
mod document;
mod error;
mod hexdump;
mod integer;
mod json;
mod relax;
mod ser;
mod yaml;

pub use color::ColorProfile;
pub use de::{from_str, Deserialize, Deserializer};
pub use doc_iter::DocPath;
pub use document::Document;
pub use error::Error;
pub use integer::{Int, IntValue};
pub use json::Json;
pub use ser::{serialize, AnnotatedSerializer};
pub use yaml::Yaml;
