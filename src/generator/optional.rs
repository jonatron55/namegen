use std::collections::HashMap;

use rand::{Rng, RngExt};

use crate::{
    generator::{Error, Generator, Result},
    styles::{ELEM, ID, PROP},
};

pub struct Optional {
    id: Option<String>,
    generator: Box<dyn Generator>,
    probability: f64,
}

impl Optional {
    pub fn new(id: Option<String>, generator: Box<dyn Generator>, probability: f64) -> Self {
        Self {
            id,
            generator,
            probability,
        }
    }
}

impl Generator for Optional {
    fn generate(&self, rand: &mut dyn Rng, hints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        let roll = if let Some(id) = self.id.as_deref()
            && let Some(hint) = hints.get(id)
        {
            match hint.parse::<bool>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(Error::InvalidHint {
                        hint: hint.to_string(),
                        id: id.to_string(),
                    });
                }
            }
        } else {
            rand.random_bool(self.probability)
        };

        if roll {
            self.generator.generate(rand, hints)
        } else {
            Ok(vec![])
        }
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!(
            "{}{ELEM}Optional{ELEM:#} {ID}{}{ID:#}:",
            indent_str,
            self.id.as_deref().unwrap_or("unnamed")
        );
        println!("{} {PROP}probability: {PROP:#}{:.2}", indent_str, self.probability);
        self.generator.analyze(verbose, indent + 2);
    }
}
