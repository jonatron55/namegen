mod capitalizer;
mod concatter;
mod markov;
mod numberer;
mod optional;
mod repeater;
mod switcher;

use std::result::Result as StdResult;

use rand::{Rng, seq::IndexedRandom};
use thiserror::Error as ThisError;

pub use capitalizer::{Capitalizer, CapitalizerMode};
pub use concatter::Concatter;
pub use markov::{Markov, Tokenizer};
pub use numberer::{NumberStyle, Numberer};
pub use optional::Optional;
pub use repeater::Repeater;
pub use switcher::Switcher;

use crate::styles::ELEM;

pub const MAX_REJECTIONS: usize = 100;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Exceeded 100 rejections during generation.")]
    MaxRejectionsExceeded,
}

pub type Result<T> = StdResult<T, Error>;

pub trait Generator {
    fn generate(&self, rng: &mut dyn Rng) -> Result<Vec<String>>;
    fn analyze(&self, verbose: bool, indent: usize);
}

impl Generator for Vec<String> {
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        Ok(self.choose(rand).map(|s| s.to_string()).into_iter().collect())
    }

    fn analyze(&self, _verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}{ELEM}Words:{ELEM:#} {} options", indent_str, self.len());
    }
}
