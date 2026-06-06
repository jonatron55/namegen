use std::{
    collections::HashMap,
    env::args,
    io::{Read, stdin},
};

use itertools::Itertools;
use lazy_static::lazy_static;
use unicode_normalization::UnicodeNormalization;

enum TengwarGlyph {
    /// Regular tengwa, mapped from a single character or digraph. These
    /// represent consonants.
    Tengwa(char),

    /// Silme tengwa, mapped from a single character or digraph. These
    /// represent sibilant sounds. This is a special case because it must be
    /// turned if the following character is a tetha.
    Silme(char, char),

    /// Tetha, mapped from a single character or dipthong. This represents a
    /// vowel and is represented as a diacritical mark above the previous
    /// consonant or a carrier if it is the first character or follows
    /// another tetha.
    Tetha(char),

    /// Punctuation, whitespace, or any other character that doesn't have a
    /// clear category.
    Other(char),
}

// These mappings use the unofficial ConScript Unicode Registry code points for
// Tengwar.
lazy_static! {
    static ref GRAPH_MAP: HashMap<char, TengwarGlyph> = {
        let mut m = HashMap::new();
        m.insert('T', TengwarGlyph::Tengwa('\u{E000}'));
        m.insert('P', TengwarGlyph::Tengwa('\u{E001}'));
        m.insert('C', TengwarGlyph::Tengwa('\u{E003}'));
        m.insert('K', TengwarGlyph::Tengwa('\u{E003}'));
        m.insert('Q', TengwarGlyph::Tengwa('\u{E003}'));
        m.insert('D', TengwarGlyph::Tengwa('\u{E004}'));
        m.insert('B', TengwarGlyph::Tengwa('\u{E005}'));
        m.insert('J', TengwarGlyph::Tengwa('\u{E006}'));
        m.insert('G', TengwarGlyph::Tengwa('\u{E007}'));
        m.insert('Þ', TengwarGlyph::Tengwa('\u{E008}'));
        m.insert('Þ', TengwarGlyph::Tengwa('\u{E008}'));
        m.insert('F', TengwarGlyph::Tengwa('\u{E009}'));
        m.insert('X', TengwarGlyph::Tengwa('\u{E00B}'));
        m.insert('Ð', TengwarGlyph::Tengwa('\u{E00C}'));
        m.insert('N', TengwarGlyph::Tengwa('\u{E010}'));
        m.insert('M', TengwarGlyph::Tengwa('\u{E011}'));
        m.insert('R', TengwarGlyph::Tengwa('\u{E014}'));
        m.insert('V', TengwarGlyph::Tengwa('\u{E015}'));
        m.insert('Y', TengwarGlyph::Tengwa('\u{E016}'));
        m.insert('W', TengwarGlyph::Tengwa('\u{E017}'));
        m.insert('L', TengwarGlyph::Tengwa('\u{E01A}'));
        m.insert('S', TengwarGlyph::Silme('\u{E024}', '\u{E025}'));
        m.insert('Z', TengwarGlyph::Silme('\u{E026}', '\u{E027}'));
        m.insert('H', TengwarGlyph::Tengwa('\u{E020}'));

        m.insert('A', TengwarGlyph::Tetha('\u{E040}'));
        m.insert('E', TengwarGlyph::Tetha('\u{E046}'));
        m.insert('I', TengwarGlyph::Tetha('\u{E044}'));
        m.insert('O', TengwarGlyph::Tetha('\u{E04A}'));
        m.insert('U', TengwarGlyph::Tetha('\u{E04C}'));

        m.insert('.', TengwarGlyph::Other('\u{E050}'));
        m.insert(';', TengwarGlyph::Other('\u{E051}'));
        m.insert('!', TengwarGlyph::Other('\u{E052}'));
        m.insert('?', TengwarGlyph::Other('\u{E053}'));
        m.insert(' ', TengwarGlyph::Other(' '));

        m
    };
    static ref DIGRAPH_MAP: HashMap<(char, char), TengwarGlyph> = {
        let mut m = HashMap::new();
        m.insert(('C', 'H'), TengwarGlyph::Tengwa('\u{E002}'));
        m.insert(('T', 'H'), TengwarGlyph::Tengwa('\u{E008}'));
        m.insert(('S', 'H'), TengwarGlyph::Tengwa('\u{E00A}'));
        m.insert(('M', 'P'), TengwarGlyph::Tengwa('\u{E00D}'));
        m.insert(('Z', 'H'), TengwarGlyph::Tengwa('\u{E00E}'));
        m.insert(('G', 'H'), TengwarGlyph::Tengwa('\u{E00F}'));
        m.insert(('N', 'Y'), TengwarGlyph::Tengwa('\u{E012}'));
        m.insert(('N', 'G'), TengwarGlyph::Tengwa('\u{E013}'));
        m.insert(('R', 'R'), TengwarGlyph::Tengwa('\u{E018}'));
        m.insert(('R', 'H'), TengwarGlyph::Tengwa('\u{E019}'));
        m.insert(('L', 'H'), TengwarGlyph::Tengwa('\u{E01B}'));
        m.insert(('S', 'S'), TengwarGlyph::Silme('\u{E026}', '\u{E027}'));
        m.insert(('H', 'W'), TengwarGlyph::Tengwa('\u{E021}'));
        m.insert(('W', 'H'), TengwarGlyph::Tengwa('\u{E021}'));
        m.insert(('A', 'A'), TengwarGlyph::Tetha('\u{E055}'));
        m.insert(('E', 'E'), TengwarGlyph::Tetha('\u{E048}'));
        m.insert(('I', 'I'), TengwarGlyph::Tetha('\u{E042}'));

        m
    };
}

const SHORT_CARRIER: char = '\u{E02E}';
const LONG_CARRIER: char = '\u{E02F}';
const DOUBLER: char = '\u{E051}';

pub fn to_tengwar(s: &str) -> String {
    let mut words = vec![];

    for s in s.split_whitespace() {
        let s = s
            .chars()
            .flat_map(|ch| ch.to_uppercase().flat_map(|ch| ch.nfd()))
            .filter(|&ch| unicode_normalization::char::canonical_combining_class(ch) == 0)
            .flat_map(|ch| {
                let (arr, len): ([char; 2], usize) = match ch {
                    'Æ' => (['A', 'E'], 2),
                    'Œ' => (['O', 'E'], 2),
                    'Ø' => (['O', '\0'], 1),
                    c => ([c, '\0'], 1),
                };
                arr.into_iter().take(len)
            });

        let mut result = String::new();
        let mut last_tengwa = None;
        let mut last_silme = None;
        let mut head: Option<char> = None;

        for next in s {
            if let Some(ch) = head {
                if let Some(glyph) = DIGRAPH_MAP.get(&(ch, next)) {
                    match_glyph(glyph, &mut last_silme, &mut last_tengwa, &mut result);
                    head = None;
                } else if let Some(glyph) = GRAPH_MAP.get(&ch) {
                    match_glyph(glyph, &mut last_silme, &mut last_tengwa, &mut result);
                    head = Some(next);
                } else {
                    unknown_glyph(ch, &mut last_silme, &mut last_tengwa, &mut result);
                    head = Some(next);
                }
            } else {
                head = Some(next);
            }
        }

        if let Some(ch) = head {
            if let Some(glyph) = GRAPH_MAP.get(&ch) {
                match_glyph(glyph, &mut last_silme, &mut last_tengwa, &mut result);
            } else {
                unknown_glyph(ch, &mut last_silme, &mut last_tengwa, &mut result);
            }
        }

        if let Some((upright, _)) = last_silme
            && upright == '\u{E024}'
        {
            result.push(match result.chars().last() {
                Some(g) => match g {
                    '\u{E000}' | '\u{E004}' | '\u{E008}' | '\u{E00C}' | '\u{E010}' | '\u{E014}' | '\u{E018}'
                    | '\u{E01C}' => '\u{E05D}',
                    '\u{E001}' | '\u{E005}' | '\u{E009}' | '\u{E00D}' | '\u{E011}' | '\u{E015}' | '\u{E019}'
                    | '\u{E01D}' => '\u{E058}',
                    '\u{E022}' => '\u{E05C}',
                    _ => upright,
                },
                _ => upright,
            });
        }

        words.push(result);
    }

    words.join(" ")
}

fn match_glyph(
    glyph: &TengwarGlyph,
    last_silme: &mut Option<(char, char)>,
    last_tengwa: &mut Option<char>,
    result: &mut String,
) {
    match glyph {
        TengwarGlyph::Tengwa(glyph) => {
            if let Some((upright, _)) = last_silme {
                result.push(*upright);
                *last_silme = None;
            }

            if let Some(g) = last_tengwa
                && g == glyph
            {
                result.push(DOUBLER);
            } else {
                result.push(*glyph);
            }

            *last_tengwa = Some(*glyph);
        }
        TengwarGlyph::Silme(upright, turned) => {
            if let Some((upright, _)) = last_silme {
                result.push(*upright);
            }
            *last_silme = Some((*upright, *turned));
            *last_tengwa = None;
        }
        TengwarGlyph::Tetha(tetha) => {
            if let Some((_, turned)) = last_silme {
                result.push(*turned);
                *last_silme = None;
            } else if let Some(_) = last_tengwa {
                *last_tengwa = None;
            } else if *tetha == '\u{E055}' || *tetha == '\u{E048}' || *tetha == '\u{E042}' {
                result.push(LONG_CARRIER);
            } else {
                result.push(SHORT_CARRIER);
            }

            result.push(*tetha);
        }
        TengwarGlyph::Other(other) => {
            if let Some((upright, _)) = last_silme {
                result.push(*upright);
                *last_silme = None;
            }

            result.push(*other);
            *last_tengwa = None;
        }
    }
}

fn unknown_glyph(ch: char, last_silme: &mut Option<(char, char)>, last_tengwa: &mut Option<char>, result: &mut String) {
    if let Some((upright, _)) = last_silme {
        result.push(*upright);
        *last_silme = None;
    }

    result.push(ch);
    *last_tengwa = None;
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

    let output = to_tengwar(&input);

    println!("{output}");
}
