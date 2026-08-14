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
