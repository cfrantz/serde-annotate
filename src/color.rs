use ansi_term::{Color, Style};

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
            aggregate: Style::new().fg(Color::Red),
            punctuation: Style::new(),
            comment: Style::new().fg(Color::White).italic(),
            null: Style::new().fg(Color::Red).bold(),
            key: Style::new().fg(Color::Cyan),
            string: Style::new().fg(Color::Green),
            escape: Style::new().fg(Color::Green).bold(),
            boolean: Style::new().fg(Color::Blue),
            integer: Style::new().fg(Color::Blue).bold(),
            float: Style::new().fg(Color::Purple),
        }
    }
}
