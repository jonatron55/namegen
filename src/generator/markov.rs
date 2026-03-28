use std::{
    collections::HashMap,
    io::{Write, stdout},
};

use rand::{Rng, RngExt};

use crate::generator::Generator;

type FreqMap = HashMap<Option<String>, i32>;

pub struct MarkovGen {
    freqs: HashMap<Option<String>, FreqMap>,
    target_len: Option<usize>,
    reject: Vec<String>,
}

impl MarkovGen {
    pub fn train(data: &[String], target_len: Option<usize>, reject: Vec<String>) -> Self {
        let mut freqs = HashMap::new();

        for word in data {
            let mut tokens = word.split('/');
            let mut token = None;
            loop {
                let next = tokens.next();

                let freq = freqs.entry(token.map(str::to_string)).or_insert_with(HashMap::new);
                *freq.entry(next.map(str::to_string)).or_insert(0) += 1;

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

        loop {
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
}
