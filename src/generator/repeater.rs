use std::collections::HashMap;

use rand::{Rng, RngExt};

use crate::{
    generator::{Error, Generator, Result},
    styles::{ELEM, ID, PROP},
};

pub struct Repeater {
    id: Option<String>,
    generator: Box<dyn Generator>,
    min: usize,
    max: usize,
}

impl Repeater {
    pub fn new(id: Option<String>, generator: Box<dyn Generator>, min: usize, max: usize) -> Self {
        Self {
            id,
            generator,
            min,
            max,
        }
    }
}

impl Generator for Repeater {
    fn generate(&self, rand: &mut dyn Rng, hints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        let count = if let Some(id) = self.id.as_deref()
            && let Some(hint) = hints.get(id)
        {
            match hint.parse::<usize>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(Error::InvalidHint {
                        hint: hint.to_string(),
                        id: id.to_string(),
                    });
                }
            }
        } else {
            rand.random_range(self.min..=self.max)
        };

        (0..count).try_fold(Vec::new(), |mut acc, _| {
            self.generator.generate(rand, hints).map(|mut v| {
                acc.append(&mut v);
                acc
            })
        })
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!(
            "{}{ELEM}Repeater{ELEM:#} {ID}{}{ID:#}:",
            indent_str,
            self.id().unwrap_or("unnamed")
        );
        println!("{} {PROP}Min: {PROP:#}{}", indent_str, self.min);
        println!("{} {PROP}Max: {PROP:#}{}", indent_str, self.max);
        self.generator.analyze(verbose, indent + 2);
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}
