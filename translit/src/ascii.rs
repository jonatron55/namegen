use std::collections::HashMap;

use lazy_static::lazy_static;
use unicode_normalization::UnicodeNormalization;

lazy_static! {
    /// This includes some common non-ASCII characters that don't decompose into
    /// ASCII equivalents (e.g. é decomposes into 'e' + '´', but 'æ' does not
    /// decompose into 'a' + 'e'). Expand as needed (only covers Latin-1 at the
    /// moment).
    pub static ref ASCII_MAP: HashMap<char, String> = {
        let mut m = HashMap::new();

        m.insert('æ', "ae".to_string());
        m.insert('Æ', "Ae".to_string());
        m.insert('ð', "th".to_string());
        m.insert('Ð', "Th".to_string());
        m.insert('ø', "o".to_string());
        m.insert('Ø', "O".to_string());
        m.insert('œ', "oe".to_string());
        m.insert('Œ', "Oe".to_string());
        m.insert('ß', "ss".to_string());
        m.insert('þ', "th".to_string());
        m.insert('Þ', "Th".to_string());

        m
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Casing {
    Snake,
    Kebab,
    Camel,
    Pascal,
    Screaming,
}

pub fn to_ascii(s: &str) -> String {
    let mut result = String::new();

    for ch in s.chars() {
        if ch.is_ascii() {
            result.push(ch);
        } else if let Some(replacement) = ASCII_MAP.get(&ch) {
            result.push_str(replacement);
        } else {
            result.push_str(&ch.nfd().filter(|ch| ch.is_ascii()).collect::<String>());
        }
    }

    result
}

pub fn to_ascii_with_casing(s: &str, casing: Casing) -> String {
    let mut result = String::with_capacity(s.len());
    let mut word_start = true;
    let mut string_start = true;

    for ch in s.chars().flat_map(|ch| ch.nfd()) {
        if ch.is_ascii_alphanumeric() {
            result.push(casing.apply_case(ch, string_start, word_start));
            word_start = false;
            string_start = false;
        } else if let Some(replacement) = ASCII_MAP.get(&ch) {
            for ch in replacement.chars() {
                result.push(casing.apply_case(ch, string_start, word_start));
                word_start = false;
                string_start = false;
            }
        } else if ch.is_whitespace() {
            if let Some(sep) = casing.separator() {
                result.push(sep);
            }

            word_start = true;
        }
    }
    result
}

impl Casing {
    fn apply_case(&self, ch: char, string_start: bool, word_start: bool) -> char {
        match self {
            Casing::Snake | Casing::Kebab => ch.to_ascii_lowercase(),
            Casing::Pascal => {
                if word_start {
                    ch.to_ascii_uppercase()
                } else {
                    ch.to_ascii_lowercase()
                }
            }
            Casing::Camel => {
                if word_start && !string_start {
                    ch.to_ascii_uppercase()
                } else {
                    ch.to_ascii_lowercase()
                }
            }
            Casing::Screaming => ch.to_ascii_uppercase(),
        }
    }

    fn separator(&self) -> Option<char> {
        match self {
            Casing::Snake | Casing::Screaming => Some('_'),
            Casing::Kebab => Some('-'),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ascii() {
        assert_eq!(to_ascii("Héllo Wørld!"), "Hello World!");
        assert_eq!(to_ascii("Æther"), "Aether");
        assert_eq!(to_ascii("ßeta"), "sseta");
        assert_eq!(to_ascii("Þorn"), "Thorn");
        assert_eq!(to_ascii("Unknown: Ω"), "Unknown: ");
    }
}
