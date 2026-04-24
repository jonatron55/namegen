mod capitalizer;
mod concatter;
mod markov;
mod numberer;
mod optional;
mod parser;
mod repeater;
mod switcher;

use std::result::Result as StdResult;

use rand::{Rng, seq::IndexedRandom};
use thiserror::Error as ThisError;

pub use parser::from_xml;

pub const MAX_REJECTIONS: usize = 100;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Exceeded 100 rejections during generation.")]
    MaxRejectionsExceeded,
}

pub type Result<T> = StdResult<T, Error>;

pub trait Generator {
    fn generate(&self, rng: &mut dyn Rng) -> Result<Vec<String>>;
    fn print_analysis(&self, indent: usize);
}

impl Generator for Vec<String> {
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        Ok(self.choose(rand).map(|s| s.to_string()).into_iter().collect())
    }

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}String selection: {} options", indent_str, self.len());
    }
}
