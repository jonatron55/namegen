use crate::generator::{Generator, Result};

use rand::Rng;
use rand::seq::IndexedRandom;

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

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("Switcher: {} subparts", self.subparts.len());
        for (i, subpart) in self.subparts.iter().enumerate() {
            print!("{} Subpart {}: ", indent_str, i);
            subpart.print_analysis(indent + 2);
        }
    }
}
