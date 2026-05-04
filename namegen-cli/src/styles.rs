use anstyle::{AnsiColor, Color, Style};

pub const ERROR: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))).bold();
pub const WARN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))).bold();
pub const PATH: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));

// pub const PUNCT: Style = Style::new().dimmed().bold();
// pub const TOKEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
// pub const ELEM: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightCyan))).bold();
// pub const ID: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))).bold();
// pub const PROP: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightMagenta))).bold();
// pub const SPEC: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))).underline().bold();
