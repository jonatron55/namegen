use std::{
    collections::HashMap,
    env::args,
    io::{Read, stdin},
};

use itertools::Itertools;
use lazy_static::lazy_static;
use unicode_normalization::UnicodeNormalization;

lazy_static! {
    static ref GRAPH_MAP: HashMap<char, &'static str> = {
        let mut m = HashMap::new();
        m.insert('F', "ᚠ");
        m.insert('U', "ᚢ");
        m.insert('V', "ᚢ");
        m.insert('Þ', "ᚦ");
        m.insert('Ð', "ᚦ");
        m.insert('O', "ᚩ");
        m.insert('R', "ᚱ");
        m.insert('C', "ᚳ");
        m.insert('K', "ᚳ");
        m.insert('Q', "ᚳ");
        m.insert('G', "ᚷ");
        m.insert('W', "ᚹ");
        m.insert('H', "ᚻ");
        m.insert('N', "ᚾ");
        m.insert('I', "ᛁ");
        m.insert('J', "ᛡ");
        m.insert('P', "ᛈ");
        m.insert('X', "ᛪ");
        m.insert('S', "ᛋ");
        m.insert('Z', "ᛋ");
        m.insert('ß', "ᛋᛋ");
        m.insert('T', "ᛏ");
        m.insert('B', "ᛒ");
        m.insert('E', "ᛖ");
        m.insert('M', "ᛗ");
        m.insert('L', "ᛚ");
        m.insert('Œ', "ᛟ");
        m.insert('Ø', "ᛟ");
        m.insert('D', "ᛞ");
        m.insert('A', "ᚪ");
        m.insert('Æ', "ᚫ");
        m.insert('Y', "ᚣ");
        m.insert(',', "᛬");
        m.insert(';', "᛬");
        m.insert('.', "᛭");
        m.insert(' ', "᛫");
        m.insert('\'', "");

        m
    };
    static ref DIGRAPH_MAP: HashMap<(char, char), &'static str> = {
        let mut m = HashMap::new();
        m.insert(('T', 'H'), "ᚦ");
        m.insert(('C', 'H'), "ᛇ");
        m.insert(('N', 'G'), "ᛝ");
        m.insert(('O', 'E'), "ᛟ");
        m.insert(('A', 'Y'), "ᚫ");
        m.insert(('E', 'A'), "ᛠ");
        m.insert(('S', 'H'), "ᛋᚳ");
        m.insert(('E', 'Y'), "ᚫ");
        m.insert(('E', 'O'), "ᛇ");
        m.insert(('I', 'O'), "ᛡ");
        m.insert(('Q', 'U'), "ᚳᚹ");

        m
    };
}

pub fn to_futhorc(s: &str) -> String {
    let s = s
        .chars()
        .flat_map(|ch| ch.to_uppercase().flat_map(|ch| ch.nfd()))
        .filter(|&ch| unicode_normalization::char::canonical_combining_class(ch) == 0);

    let mut result = String::new();
    let mut prev: Option<char> = None;

    for ch in s {
        if let Some(p) = prev {
            if let Some(digraph) = DIGRAPH_MAP.get(&(p, ch)) {
                result.push_str(digraph);
                prev = None;
            } else {
                if let Some(mapped) = GRAPH_MAP.get(&p) {
                    result.push_str(mapped);
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
            result.push_str(mapped);
        } else {
            result.push(p);
        }
    }

    result
}

#[allow(unused)]
fn main() {
    let input = if args().len() > 1 {
        #[allow(unstable_name_collisions)]
        args().skip(1).intersperse(" ".to_string()).collect()
    } else {
        let mut buf = String::new();
        stdin().read_to_string(&mut buf).unwrap();
        buf
    };

    let output = to_futhorc(&input);

    println!("{output}");
}
