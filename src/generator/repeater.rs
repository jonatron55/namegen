use crate::generator::{Generator, Result};

use rand::{Rng, RngExt};

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

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("Repeater: min {}, max {}", self.min, self.max);
        print!("{} Subpart: ", indent_str);
        self.generator.print_analysis(indent + 2);
    }
}
