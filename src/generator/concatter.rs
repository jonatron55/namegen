use itertools::Itertools;
use rand::Rng;

use crate::generator::{Error, Generator, MAX_REJECTIONS, Result};

pub struct Concatter {
    subparts: Vec<Box<dyn Generator>>,
    joiner: String,
    reject: Vec<String>,
}

impl Concatter {
    pub fn new(subparts: Vec<Box<dyn Generator>>, reject: Vec<String>) -> Self {
        Self {
            subparts,
            joiner: "".to_string(),
            reject,
        }
    }

    pub fn with_joiner(mut self, joiner: String) -> Self {
        self.joiner = joiner;
        self
    }
}

impl Generator for Concatter {
    #[allow(unstable_name_collisions)] // `intersperse` may at some point be incorporated into the standard library, but for now we need to use the one from itertools
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        let mut attempt = 0;

        loop {
            if attempt > MAX_REJECTIONS {
                return Err(Error::MaxRejectionsExceeded);
            }
            attempt += 1;

            let name: Vec<String> = self.subparts.iter().try_fold(Vec::new(), |mut acc, part| {
                part.generate(rand).map(|mut v| {
                    acc.append(&mut v);
                    acc
                })
            })?;

            // Reject duplicates
            if name.iter().tuple_windows().any(|(a, b)| a == b) {
                continue;
            }

            let name = name.join(&self.joiner);
            if !self.reject.contains(&name) {
                return Ok(vec![name]);
            }
        }
    }

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}Concatinator: {} subparts", indent_str, self.subparts.len());
        for (i, subpart) in self.subparts.iter().enumerate() {
            print!("{} Subpart {}: ", indent_str, i);
            subpart.print_analysis(indent + 2);
        }
    }
}
