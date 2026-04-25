mod tokenizer;

use std::{
    borrow::Cow,
    collections::HashMap,
    io::{Write, stdout},
};

use rand::{Rng, RngExt};

use crate::{
    generator::{Error, Generator, MAX_REJECTIONS, Result},
    styles::{ELEM, PROP, PUNCT, SPEC, TOKEN},
};

pub use tokenizer::Tokenizer;

type FreqMap = HashMap<Option<String>, i32>;

/// Maximum length of generated string to prevent infinite loops in cyclic or near-cyclic models.
pub const MAX_LEN: usize = 100;

pub struct MarkovGen {
    freqs: HashMap<Option<String>, FreqMap>,
    target_len: Option<usize>,
    cutoff_len: Option<usize>,
    reject: Vec<String>,
}

impl MarkovGen {
    pub fn train(
        data: &[impl AsRef<str>],
        target_len: Option<usize>,
        cutoff_len: Option<usize>,
        reject: Vec<String>,
        tokenizer: &Tokenizer,
        uniform: bool,
    ) -> Self {
        let mut freqs = HashMap::new();

        for word in data.iter() {
            let mut tokens = tokenizer.tokenize(word.as_ref()).into_iter();
            let mut token = None;
            loop {
                let next = tokens.next();

                let freq = freqs.entry(token.map(str::to_string)).or_insert_with(HashMap::new);
                if uniform {
                    freq.entry(next.map(str::to_string)).or_insert(1);
                } else {
                    *freq.entry(next.map(str::to_string)).or_insert(0) += 1;
                }

                token = next;

                if token.is_none() {
                    break;
                }
            }
        }

        Self {
            freqs,
            target_len,
            cutoff_len,
            reject,
        }
    }
}

impl Generator for MarkovGen {
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        let mut name = String::new();
        let mut token: Option<String> = None;
        let mut attempt = 0;

        loop {
            if attempt > MAX_REJECTIONS {
                return Err(Error::MaxRejectionsExceeded);
            }

            'inner: loop {
                if name.len() >= MAX_LEN {
                    // Safety valve to prevent infinite loops in degenerate models
                    return Ok(vec![name]);
                }

                let freq = self.freqs.get(&token).unwrap();

                // Stop if we hit the cutoff length and there's a valid halt option
                if let Some(cutoff_len) = self.cutoff_len
                    && name.len() >= cutoff_len
                    && freq.contains_key(&None)
                {
                    return Ok(vec![name]);
                }

                // If we're under the target length and there are some valid continuations,
                // remove the halt option to avoid early termination
                let freq = if let Some(target_len) = self.target_len
                    && name.len() < target_len
                    && freq.keys().any(|k| k.is_some())
                {
                    let mut freq = freq.clone();
                    freq.remove(&None);
                    Cow::Owned(freq)
                } else {
                    Cow::Borrowed(freq)
                };

                let total: i32 = freq.values().sum();
                let mut roll: i32 = rand.random_range(0..total);

                for (next, count) in freq.iter() {
                    roll -= count;
                    if roll < 0 {
                        if let Some(next) = next {
                            stdout().flush().unwrap();
                            name.push_str(next);
                            token = Some(next.to_string());
                            break;
                        } else {
                            if self.reject.contains(&name) {
                                attempt += 1;
                                break 'inner;
                            } else {
                                return Ok(vec![name]);
                            }
                        }
                    }
                }

                if token.is_none() {
                    break;
                }
            }

            name.clear();
            token = None;
        }
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        // Branching metrics
        let mut total_succ = 0usize;
        let mut weighted_entropy = 0.0f64;
        let mut grand_total = 0i64;
        let mut dead_ends = 0usize;

        for freq in self.freqs.values() {
            let n = freq.len();
            total_succ += n;
            if n == 1 && freq.contains_key(&None) {
                dead_ends += 1;
            }

            let total: i32 = freq.values().sum();
            grand_total += total as i64;

            // H(s) in bits
            let h: f64 = freq
                .values()
                .map(|&c| {
                    let p = c as f64 / total as f64;
                    if p > 0.0 { -p * p.log2() } else { 0.0 }
                })
                .sum();

            // weight by state frequency (outgoing count as proxy)
            weighted_entropy += h * total as f64;
        }

        let avg_branching = total_succ as f64 / self.freqs.len() as f64;
        let avg_entropy = weighted_entropy / grand_total as f64;
        let perplexity = avg_entropy.exp2();

        let indent_str = " ".repeat(indent);

        println!(
            "{}{ELEM}Markov generator:{ELEM:#} {} states",
            indent_str,
            self.freqs.len()
        );

        println!(
            "{} {PROP}Average branching factor: {PROP:#}{:.2}",
            indent_str, avg_branching
        );
        println!(
            "{} {PROP}Average entropy (bits):   {PROP:#}{:.2}",
            indent_str, avg_entropy
        );
        println!(
            "{} {PROP}Perplexity:               {PROP:#}{:.2}",
            indent_str, perplexity
        );
        println!(
            "{} {PROP}Dead-end states:          {PROP:#}{} / {}",
            indent_str,
            dead_ends,
            self.freqs.len()
        );

        if verbose {
            println!("{} ---", indent_str);
            for (token, freq) in &self.freqs {
                print!("{} ", indent_str);

                if let Some(token) = token {
                    print!("{TOKEN}\"{token}\"{TOKEN:#} {PUNCT}->{PUNCT:#} ");
                } else {
                    print!("{SPEC}BEGIN{SPEC:#} {PUNCT}->{PUNCT:#} ");
                }
                for (next, count) in freq {
                    if let Some(next) = next {
                        print!("[{TOKEN}\"{next}\"{TOKEN:#}{PUNCT}: {count}{PUNCT:#}], ");
                    } else {
                        print!("[{SPEC}HALT{SPEC:#}{PUNCT}: {count}{PUNCT:#}], ");
                    }
                }
                println!();
            }
        }
    }
}
