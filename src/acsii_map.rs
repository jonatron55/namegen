use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref ASCII_MAP: HashMap<char, String> = {
        let mut m = HashMap::new();
        for c in ' '..='~' {
            m.insert(c, c.to_string());
        }

        m.insert('À', "A".to_string());
        m.insert('Á', "A".to_string());
        m.insert('Â', "A".to_string());
        m.insert('Ã', "A".to_string());
        m.insert('Ä', "A".to_string());
        m.insert('Å', "A".to_string());
        m.insert('Æ', "Ae".to_string());
        m.insert('È', "E".to_string());
        m.insert('É', "E".to_string());
        m.insert('Ê', "E".to_string());
        m.insert('Ë', "E".to_string());
        m.insert('Ì', "I".to_string());
        m.insert('Í', "I".to_string());
        m.insert('Î', "I".to_string());
        m.insert('Ï', "I".to_string());
        m.insert('Ò', "O".to_string());
        m.insert('Ó', "O".to_string());
        m.insert('Ô', "O".to_string());
        m.insert('Õ', "O".to_string());
        m.insert('Ö', "O".to_string());
        m.insert('Ø', "O".to_string());
        m.insert('Œ', "Oe".to_string());
        m.insert('Ù', "U".to_string());
        m.insert('Ú', "U".to_string());
        m.insert('Û', "U".to_string());
        m.insert('Ü', "U".to_string());
        m.insert('Ý', "Y".to_string());
        m.insert('à', "a".to_string());
        m.insert('á', "a".to_string());
        m.insert('â', "a".to_string());
        m.insert('ã', "a".to_string());
        m.insert('ä', "a".to_string());
        m.insert('å', "a".to_string());
        m.insert('æ', "ae".to_string());
        m.insert('è', "e".to_string());
        m.insert('é', "e".to_string());
        m.insert('ê', "e".to_string());
        m.insert('ë', "e".to_string());
        m.insert('ì', "i".to_string());
        m.insert('í', "i".to_string());
        m.insert('î', "i".to_string());
        m.insert('ï', "i".to_string());
        m.insert('ò', "o".to_string());
        m.insert('ó', "o".to_string());
        m.insert('ô', "o".to_string());
        m.insert('õ', "o".to_string());
        m.insert('ö', "o".to_string());
        m.insert('ø', "o".to_string());
        m.insert('œ', "oe".to_string());
        m.insert('ù', "u".to_string());
        m.insert('ú', "u".to_string());
        m.insert('û', "u".to_string());
        m.insert('ü', "u".to_string());
        m.insert('ý', "y".to_string());
        m.insert('ÿ', "y".to_string());
        m.insert('Ů', "U".to_string());
        m.insert('ů', "u".to_string());
        m.insert('Ÿ', "Y".to_string());
        m.insert('Ř', "R".to_string());
        m.insert('ř', "r".to_string());
        m.insert('Ç', "C".to_string());
        m.insert('Ð', "Th".to_string());
        m.insert('Ñ', "N".to_string());
        m.insert('Þ', "Th".to_string());
        m.insert('ß', "ss".to_string());
        m.insert('ç', "c".to_string());
        m.insert('ð', "th".to_string());
        m.insert('ñ', "n".to_string());
        m.insert('þ', "th".to_string());
        m.insert('Ň', "N".to_string());
        m.insert('ň', "n".to_string());
        m.insert('Š', "S".to_string());
        m.insert('š', "s".to_string());
        m.insert('Ž', "Z".to_string());
        m.insert('ž', "z".to_string());
        m.insert('Č', "C".to_string());
        m.insert('č', "c".to_string());
        m.insert('Ď', "D".to_string());
        m.insert('ď', "d".to_string());
        m.insert('Ť', "T".to_string());
        m.insert('ť', "t".to_string());

        m
    };
}

pub fn to_ascii(s: &str) -> String {
    s.chars().map(|c| ASCII_MAP.get(&c).map(|s| s.as_str()).unwrap_or("")).collect()
}
