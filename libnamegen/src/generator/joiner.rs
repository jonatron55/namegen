use std::collections::HashMap;

use itertools::Itertools;
use rand::Rng;

use crate::generator::{Error, Generator, MAX_REJECTIONS, Result};

pub struct Joiner {
    id: Option<String>,
    subparts: Vec<Box<dyn Generator>>,
    sep: String,
    reject: Vec<String>,
}

impl Joiner {
    pub fn new(id: Option<String>, subparts: Vec<Box<dyn Generator>>, sep: String, reject: Vec<String>) -> Self {
        Self {
            id,
            subparts,
            sep,
            reject,
        }
    }
}

impl Generator for Joiner {
    #[allow(unstable_name_collisions)] // `intersperse` may at some point be incorporated into the standard library, but for now we need to use the one from itertools
    fn generate(&self, rand: &mut dyn Rng, constraints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            return Err(Error::InvalidHint {
                constraint: constraint.to_string(),
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
                part.generate(rand, constraints).map(|mut v| {
                    acc.append(&mut v);
                    acc
                })
            })?;

            // Reject duplicates
            if name.iter().tuple_windows().any(|(a, b)| a == b) {
                continue;
            }

            let name = name.join(&self.sep);
            if !self.reject.contains(&name) {
                return Ok(vec![name]);
            }
        }
    }
}
