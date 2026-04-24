mod concatter;
mod markov;
mod numberer;
mod optional;
mod parser;
mod repeater;
mod switcher;

pub use parser::from_xml;

use rand::{Rng, seq::IndexedRandom};

pub trait Generator {
    fn generate(&self, rng: &mut dyn Rng) -> Vec<String>;
    fn print_analysis(&self, indent: usize);
}

impl Generator for Vec<String> {
    fn generate(&self, rand: &mut dyn Rng) -> Vec<String> {
        self.choose(rand).map(|s| s.to_string()).into_iter().collect()
    }

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}String selection: {} options", indent_str, self.len());
    }
}
