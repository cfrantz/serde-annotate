use ansi_term::{Color, Style};

#[derive(Default, Clone, Copy)]
pub struct ColorProfile {
    pub aggregate: Style,
    pub punctuation: Style,
    pub comment: Style,
    pub null: Style,
    pub key: Style,
    pub string: Style,
    pub escape: Style,
    pub boolean: Style,
    pub integer: Style,
    pub float: Style,
}

impl ColorProfile {
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
