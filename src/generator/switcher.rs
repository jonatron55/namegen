use std::collections::HashMap;

use rand::Rng;
use rand::seq::IndexedRandom;

use crate::{
    generator::{Error, Generator, Result},
    styles::{ELEM, ID},
};

pub struct Switcher {
    id: Option<String>,
    subparts: Vec<Box<dyn Generator>>,
}

impl Switcher {
    pub fn new(id: Option<String>, subparts: Vec<Box<dyn Generator>>) -> Self {
        Self { id, subparts }
    }
}

impl Generator for Switcher {
    fn generate(&self, rand: &mut dyn Rng, hints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        if let Some(id) = self.id.as_deref()
            && let Some(hint) = hints.get(id)
        {
            let idx = hint.parse::<usize>().map_err(|_| Error::InvalidHint {
                id: id.to_string(),
                hint: hint.to_string(),
            })?;
            if idx < self.subparts.len() {
                return self.subparts[idx].generate(rand, hints);
            } else {
                return Err(Error::InvalidHint {
                    id: id.to_string(),
                    hint: hint.to_string(),
                });
            }
        }

        self.subparts
            .choose(rand)
            .map(|part| part.generate(rand, hints))
            .unwrap_or_else(|| Ok(vec![]))
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!(
            "{}{ELEM}Switcher{ELEM:#} {ID}{}{ID:#}: {} subparts",
            indent_str,
            self.id.as_deref().unwrap_or("unnamed"),
            self.subparts.len()
        );
        for subpart in self.subparts.iter() {
            subpart.analyze(verbose, indent + 2);
        }
    }
}
