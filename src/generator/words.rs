use std::collections::HashMap;

use rand::{
    Rng,
    seq::{IndexedRandom, IteratorRandom},
};

use crate::{
    generator::{Error, Generator, Result},
    styles::{ELEM, ID},
};

pub struct Words {
    id: Option<String>,
    words: Vec<String>,
}

impl Words {
    pub fn new(id: Option<String>, words: Vec<String>) -> Self {
        Self { id, words }
    }
}

impl Generator for Words {
    fn generate(&self, rand: &mut dyn Rng, constraints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            let words = self.words.iter().filter(|w| w.starts_with(constraint));
            if words.clone().count() == 0 {
                return Err(Error::Overconstrained { id: id.to_string() });
            }

            Ok(words.choose(rand).map(|s| s.to_string()).into_iter().collect())
        } else {
            Ok(self.words.choose(rand).map(|s| s.to_string()).into_iter().collect())
        }
    }

    fn analyze(&self, _verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!(
            "{}{ELEM}Words{ELEM:#} {ID}{}{ID:#}: {} words",
            indent_str,
            self.id.as_deref().unwrap_or("unnamed"),
            self.words.len()
        );
    }
}
