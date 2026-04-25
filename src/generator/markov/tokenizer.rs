use std::collections::HashMap;

/// Sonority rank assigned to vowel nuclei. Chars at this rank act as syllable
/// peaks.
pub const RANK_VOWEL: u8 = 5;
/// Sonority rank for glides (y, w).
pub const RANK_GLIDE: u8 = 4;
/// Sonority rank for liquids (l, r).
pub const RANK_LIQUID: u8 = 3;
/// Sonority rank for nasals and fricatives.
pub const RANK_NASAL_FRICATIVE: u8 = 2;
/// Sonority rank for stops and affricates.
pub const RANK_STOP: u8 = 1;

/// Tokenizer options for Markov generator.
///
/// The tokenizer is used to split training data into units for the Markov
/// chain.
pub enum Tokenizer {
    /// Split on any of the specified chars.
    ///
    /// This is effectively manually specifying the tokenization of the training
    /// data. It is a tedious option to set up, but allows for arbitrary
    /// tokenization and fine control over the tokens.
    ///
    /// Consecutive delimiters yield empty tokens.
    SplitChars(Vec<char>),

    /// Split into tokens of the specified length.
    ///
    /// The final token may be shorter if the word length is not a multiple of
    /// the specified length.
    Chunker(usize),

    /// Sonority Sequencing Principle tokenizer.
    ///
    /// Each char is assigned a rank; vowels (rank 5) act as syllable peaks, and
    /// syllables are split at the lowest-sonority position between adjacent
    /// peaks (earliest on ties → maximum-onset). Chars not present in `ranks`
    /// are treated as separators and emitted as single-char tokens.
    Ssp { ranks: HashMap<char, u8> },
}

impl Tokenizer {
    pub fn default_ssp() -> Self {
        let mut ranks = HashMap::new();
        let mut add = |chars: &str, rank: u8| {
            for c in chars.chars() {
                ranks.insert(c, rank);
            }
        };

        add(
            "aAáÁàÀâÂåÅäÄãÃæÆeEéÉèÈêÊëËiIíÍìÌîÎïÏoOóÓòÒôÔöÖõÕøØuUúÚùÙûÛůŮüÜyYýÝÿŸ",
            RANK_VOWEL,
        );
        add("wW", RANK_GLIDE);
        add("lLrRřŘ", RANK_LIQUID);
        add("çÇðÐfFhHmMnNňŇñÑsSšŠßvVzZžŽþÞ", RANK_NASAL_FRICATIVE);
        add("bBcCčČdDďĎgGjJkKpPqQtTťŤxX", RANK_STOP);

        Self::Ssp { ranks }
    }

    pub fn tokenize<'a>(&self, word: &'a str) -> Vec<&'a str> {
        match self {
            Self::SplitChars(c) => word.split(|ch| c.contains(&ch)).collect(),
            Self::Chunker(len) => {
                let len = *len;
                let mut tokens = Vec::new();
                let mut start = 0;
                let mut count = 0;
                for (i, _) in word.char_indices() {
                    if count == len {
                        tokens.push(&word[start..i]);
                        start = i;
                        count = 0;
                    }
                    count += 1;
                }
                if start < word.len() {
                    tokens.push(&word[start..]);
                }
                tokens
            }
            Self::Ssp { ranks } => {
                let mut tokens = Vec::new();
                let mut token_start: Option<usize> = None;
                let mut last_peak: Option<usize> = None;
                let mut min_since_peak: Option<(usize, u8)> = None;

                for (i, c) in word.char_indices() {
                    match ranks.get(&c).copied() {
                        None => {
                            // Separator character (not a letter in one of the ranks).
                            // Emit as single-char token and split any current token.
                            if let Some(s) = token_start {
                                tokens.push(&word[s..i]);
                            }
                            tokens.push(&word[i..i + c.len_utf8()]);
                            token_start = None;
                            last_peak = None;
                            min_since_peak = None;
                        }
                        Some(rank) => {
                            if token_start.is_none() {
                                token_start = Some(i);
                            }
                            if rank == RANK_VOWEL {
                                if last_peak.is_some() {
                                    // Split between previous peak and this one at the
                                    // lowest-sonority position (or immediately before
                                    // this vowel if there were no intervening consonants).
                                    let split = min_since_peak.map_or(i, |(p, _)| p);
                                    if let Some(s) = token_start
                                        && split > s
                                    {
                                        tokens.push(&word[s..split]);
                                        token_start = Some(split);
                                    }
                                }
                                last_peak = Some(i);
                                min_since_peak = None;
                            } else if last_peak.is_some() {
                                // Track minimum sonority; keep earliest offset on ties
                                // so the next syllable gets the longer (maximum) onset.
                                match min_since_peak {
                                    None => min_since_peak = Some((i, rank)),
                                    Some((_, r)) if rank < r => min_since_peak = Some((i, rank)),
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                if let Some(s) = token_start {
                    tokens.push(&word[s..]);
                }

                tokens
            }
        }
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::default_ssp()
    }
}
