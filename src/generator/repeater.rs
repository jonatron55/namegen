use crate::generator::Generator;

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
    fn generate(&self, rand: &mut dyn Rng) -> Vec<String> {
        let count = rand.random_range(self.min..=self.max);
        (0..count).flat_map(|_| self.generator.generate(rand)).collect()
    }

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("Repeater: min {}, max {}", self.min, self.max);
        print!("{} Subpart: ", indent_str);
        self.generator.print_analysis(indent + 2);
    }
}
