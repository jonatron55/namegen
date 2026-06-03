use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;
use unicode_normalization::UnicodeNormalization;

lazy_static! {
    pub static ref GRAPH_MAP: HashMap<char, char> = {
        let mut m = HashMap::new();
        m.insert('F', 'ᚠ');
        m.insert('U', 'ᚢ');
        m.insert('Þ', 'ᚦ');
        m.insert('Ð', 'ᚦ');
        m.insert('O', 'ᚩ');
        m.insert('R', 'ᚱ');
        m.insert('C', 'ᚳ');
        m.insert('K', 'ᚳ');
        m.insert('G', 'ᚷ');
        m.insert('W', 'ᚹ');
        m.insert('H', 'ᚻ');
        m.insert('N', 'ᚾ');
        m.insert('I', 'ᛁ');
        m.insert('J', 'ᛡ');
        m.insert('P', 'ᛈ');
        m.insert('X', 'ᛪ');
        m.insert('S', 'ᛋ');
        m.insert('Z', 'ᛋ');
        m.insert('T', 'ᛏ');
        m.insert('B', 'ᛒ');
        m.insert('E', 'ᛖ');
        m.insert('M', 'ᛗ');
        m.insert('L', 'ᛚ');
        m.insert('Œ', 'ᛟ');
        m.insert('Ø', 'ᛟ');
        m.insert('D', 'ᛞ');
        m.insert('A', 'ᚪ');
        m.insert('Æ', 'ᚫ');
        m.insert('Y', 'ᚣ');

        m
    };
    pub static ref DIGRAPH_MAP: HashMap<(char, char), String> = {
        let mut m = HashMap::new();
        m.insert(('T', 'H'), "ᚦ".to_string());
        m.insert(('C', 'H'), "ᛇ".to_string());
        m.insert(('N', 'G'), "ᛝ".to_string());
        m.insert(('O', 'E'), "ᛟ".to_string());
        m.insert(('A', 'Y'), "ᚫ".to_string());
        m.insert(('E', 'A'), "ᛠ".to_string());
        m.insert(('S', 'H'), "ᛋᚳ".to_string());
        m.insert(('E', 'Y'), "ᚫ".to_string());
        m.insert(('E', 'O'), "ᛇ".to_string());
        m.insert(('I', 'O'), "ᛡ".to_string());

        m
    };
    pub static ref NORMALIZED_CHARS: HashSet<char> = {
        HashSet::from([
            'A', 'Æ', 'B', 'C', 'D', 'Ð', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'Ø', 'Œ', 'P', 'R', 'S',
            'T', 'U', 'W', 'X', 'Y', 'Z', 'Þ',
        ])
    };
}

pub fn to_futhark(s: &str) -> String {
    let s = s
        .chars()
        .flat_map(|ch| ch.to_uppercase().flat_map(|ch| ch.nfd()))
        .filter(|ch| NORMALIZED_CHARS.contains(ch));

    let mut result = String::new();
    let mut prev: Option<char> = None;

    for ch in s {
        if let Some(p) = prev {
            if let Some(digraph) = DIGRAPH_MAP.get(&(p, ch)) {
                result.push_str(digraph);
                prev = None;
            } else {
                if let Some(mapped) = GRAPH_MAP.get(&p) {
                    result.push(*mapped);
                } else {
                    result.push(p);
                }
                prev = Some(ch);
            }
        } else {
            prev = Some(ch);
        }
    }

    if let Some(p) = prev {
        if let Some(mapped) = GRAPH_MAP.get(&p) {
            result.push(*mapped);
        } else {
            result.push(p);
        }
    }

    result
}
