use std::collections::HashMap;

use itertools::Itertools;
use rand::Rng;

use crate::{
    generator::{Error, Generator, MAX_REJECTIONS, Result},
    styles::{ELEM, ID},
};

pub struct Concatter {
    id: Option<String>,
    subparts: Vec<Box<dyn Generator>>,
    joiner: String,
    reject: Vec<String>,
}

impl Concatter {
    pub fn new(id: Option<String>, subparts: Vec<Box<dyn Generator>>, joiner: String, reject: Vec<String>) -> Self {
        Self {
            id,
            subparts,
            joiner,
            reject,
        }
    }
}

impl Generator for Concatter {
    #[allow(unstable_name_collisions)] // `intersperse` may at some point be incorporated into the standard library, but for now we need to use the one from itertools
    fn generate(&self, rand: &mut dyn Rng, hints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        if let Some(id) = self.id.as_deref()
            && let Some(hint) = hints.get(id)
        {
            return Err(Error::InvalidHint {
                hint: hint.to_string(),
                id: id.to_string(),
            });
        }

        let mut attempt = 0;

        loop {
            if attempt > MAX_REJECTIONS {
                return Err(Error::MaxRejectionsExceeded);
            }
            attempt += 1;

            let name: Vec<String> = self.subparts.iter().try_fold(Vec::new(), |mut acc, part| {
                part.generate(rand, hints).map(|mut v| {
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
            "{}{ELEM}Concatter{ELEM:#} {ID}{}{ID:#}: {} subparts",
            indent_str,
            self.id.as_deref().unwrap_or("unnamed"),
            self.subparts.len()
        );

        for subpart in self.subparts.iter() {
            subpart.analyze(verbose, indent + 2);
        }
    }
}
