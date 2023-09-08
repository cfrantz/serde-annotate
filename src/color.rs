use std::fmt::Display;

use anstyle::{AnsiColor, Style};

/// A `ColorProfile` describes how to apply color information when rendering a document.
#[derive(Default, Clone, Copy)]
pub struct ColorProfile {
    /// The style to use for aggregate symbols (`[]{}`).
    pub aggregate: Style,
    /// The style to use for punctuation symbols (`"',`).
    pub punctuation: Style,
    /// The style to use for comments.
    pub comment: Style,
    /// The style to use for null values.
    pub null: Style,
    /// The style to use for object keys.
    pub key: Style,
    /// The style to use for string values.
    pub string: Style,
    /// The style to use for escap sequences in strings.
    pub escape: Style,
    /// The style to use for boolean values.
    pub boolean: Style,
    /// The style to use for integer values.
    pub integer: Style,
    /// The style to use for float values.
    pub float: Style,
}

impl ColorProfile {
    /// Returns a basic color profile.
    pub fn basic() -> Self {
        ColorProfile {
            aggregate: AnsiColor::Red.on_default(),
            punctuation: Style::new(),
            comment: AnsiColor::White.on_default().italic(),
            null: AnsiColor::Red.on_default().bold(),
            key: AnsiColor::Cyan.on_default(),
            string: AnsiColor::Green.on_default(),
            escape: AnsiColor::Green.on_default().bold(),
            boolean: AnsiColor::Blue.on_default(),
            integer: AnsiColor::Blue.on_default().bold(),
            float: AnsiColor::Magenta.on_default(),
        }
    }
}

pub(crate) struct Paint<T> {
    style: Style,
    text: T,
}

impl<T: Display> std::fmt::Display for Paint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.style.render(),
            self.text,
            self.style.render_reset()
        )
    }
}

pub(crate) trait PaintExt {
    fn paint<T>(self, text: T) -> Paint<T>;
}

impl PaintExt for Style {
    fn paint<T>(self, text: T) -> Paint<T> {
        Paint { style: self, text }
    }
}
