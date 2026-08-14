use anstyle::{AnsiColor, Color, Style};

pub const ERROR: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))).bold();
pub const PATH: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
