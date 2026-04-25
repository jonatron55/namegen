use rand::{Rng, RngExt};

use crate::{
    generator::{Generator, Result},
    styles::{ELEM, PROP},
};

pub struct Repeater {
    generator: Box<dyn Generator>,
    min: usize,
    max: usize,
}

impl Repeater {
    pub fn new(generator: Box<dyn Generator>, min: usize, max: usize) -> Self {
        Self { generator, min, max }
    }
}

impl Generator for Repeater {
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        let count = rand.random_range(self.min..=self.max);
        (0..count).try_fold(Vec::new(), |mut acc, _| {
            self.generator.generate(rand).map(|mut v| {
                acc.append(&mut v);
                acc
            })
        })
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}{ELEM}Repeater:{ELEM:#}", indent_str);
        println!("{} {PROP}Min: {PROP:#}{}", indent_str, self.min);
        println!("{} {PROP}Max: {PROP:#}{}", indent_str, self.max);
        self.generator.analyze(verbose, indent + 2);
    }
}
