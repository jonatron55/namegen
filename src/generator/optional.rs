use rand::{Rng, RngExt};

use crate::{
    generator::{Generator, Result},
    styles::{ELEM, PROP},
};

pub struct Optional {
    generator: Box<dyn Generator>,
    probability: f64,
}

impl Optional {
    pub fn new(generator: Box<dyn Generator>, probability: f64) -> Self {
        Self { generator, probability }
    }
}

impl Generator for Optional {
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        if rand.random_bool(self.probability) {
            self.generator.generate(rand)
        } else {
            Ok(vec![])
        }
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}{ELEM}Optional:{ELEM:#} ", indent_str);
        println!("{} {PROP}probability: {PROP:#}{:.2}", indent_str, self.probability);
        self.generator.analyze(verbose, indent + 2);
    }
}
