use itertools::Itertools;
use rand::Rng;

use crate::{
    generator::{Error, Generator, MAX_REJECTIONS, Result},
    styles::ELEM,
};

pub struct Concatter {
    subparts: Vec<Box<dyn Generator>>,
    joiner: String,
    reject: Vec<String>,
}

impl Concatter {
    pub fn new(subparts: Vec<Box<dyn Generator>>, joiner: String, reject: Vec<String>) -> Self {
        Self {
            subparts,
            joiner,
            reject,
        }
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

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!(
            "{}{ELEM}Concatter:{ELEM:#} {} subparts",
            indent_str,
            self.subparts.len()
        );

        for subpart in self.subparts.iter() {
            subpart.analyze(verbose, indent + 2);
        }
    }
}
