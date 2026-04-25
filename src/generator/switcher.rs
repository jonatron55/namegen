use rand::Rng;
use rand::seq::IndexedRandom;

use crate::{
    generator::{Generator, Result},
    styles::ELEM,
};

pub struct Switcher {
    subparts: Vec<Box<dyn Generator>>,
}

impl Switcher {
    pub fn new(subparts: Vec<Box<dyn Generator>>) -> Self {
        Self { subparts }
    }
}

impl Generator for Switcher {
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        self.subparts
            .choose(rand)
            .map(|part| part.generate(rand))
            .unwrap_or_else(|| Ok(vec![]))
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}{ELEM}Switcher:{ELEM:#} {} subparts", indent_str, self.subparts.len());
        for subpart in self.subparts.iter() {
            subpart.analyze(verbose, indent + 2);
        }
    }
}
