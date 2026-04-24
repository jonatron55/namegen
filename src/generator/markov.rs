mod tokenizer;

use std::{
    collections::HashMap,
    io::{Write, stdout},
};

use rand::{Rng, RngExt};

use crate::generator::Generator;

pub use tokenizer::Tokenizer;

type FreqMap = HashMap<Option<String>, i32>;

pub struct MarkovGen {
    freqs: HashMap<Option<String>, FreqMap>,
    target_len: Option<usize>,
    reject: Vec<String>,
}

const MAX_ATTEMPTS: usize = 100;

impl MarkovGen {
    pub fn train(
        data: &[String],
        target_len: Option<usize>,
        reject: Vec<String>,
        tokenizer: &Tokenizer,
        uniform: bool,
    ) -> Self {
        let mut freqs = HashMap::new();

        for word in data.iter() {
            let mut tokens = tokenizer.tokenize(word).into_iter();
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
            reject,
        }
    }
}

impl Generator for MarkovGen {
    fn generate(&self, rand: &mut dyn Rng) -> Vec<String> {
        let mut name = String::new();
        let mut token: Option<String> = None;
        let mut attempt = 0;

        loop {
            if attempt >= MAX_ATTEMPTS {
                panic!("Markov generation failed after {} attempts", MAX_ATTEMPTS);
            }

            'inner: loop {
                let freq = self.freqs.get(&token).unwrap();
                let total: i32 = freq.values().sum();
                let mut roll: i32 = rand.random_range(0..total);

                if let Some(target_len) = self.target_len
                    && name.len() >= target_len
                    && freq.contains_key(&None)
                {
                    return vec![name];
                }

                for (next, count) in freq.iter() {
                    roll -= count;
                    if roll < 0 {
                        if let Some(next) = next {
                            stdout().flush().unwrap();
                            name.push_str(next);
                            token = Some(next.clone());
                            break;
                        } else {
                            if self.reject.contains(&name) {
                                attempt += 1;
                                break 'inner;
                            } else {
                                return vec![name];
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

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("Markov generator: {} states", self.freqs.len());

        for (token, freq) in &self.freqs {
            print!("{} ", indent_str);

            if let Some(token) = token {
                print!("\"{token}\" -> ");
            } else {
                print!("BEGIN -> ");
            }
            for (next, count) in freq {
                if let Some(next) = next {
                    print!("[\"{next}\": {count}], ");
                } else {
                    print!("[HALT: {count}], ");
                }
            }
            println!();
        }

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

        println!("{}Average branching factor: {:.2}", indent_str, avg_branching);
        println!("{}Average entropy (bits):   {:.2}", indent_str, avg_entropy);
        println!("{}Perplexity:           {:.2}", indent_str, perplexity);
        println!(
            "{}Dead-end states:      {} / {}",
            indent_str,
            dead_ends,
            self.freqs.len()
        );
    }
}
