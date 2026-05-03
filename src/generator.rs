mod capitalizer;
mod concatter;
mod literal;
mod markov;
mod matcher;
mod numberer;
mod optional;
mod repeater;
mod switcher;
mod words;

use std::{collections::HashMap, result::Result as StdResult};

use rand::Rng;
use thiserror::Error as ThisError;

pub use capitalizer::{Capitalizer, CapitalizerMode};
pub use concatter::Concatter;
pub use literal::Literal;
pub use markov::{Markov, Tokenizer};
pub use matcher::Matcher;
pub use numberer::{NumberStyle, Numberer};
pub use optional::Optional;
pub use repeater::Repeater;
pub use switcher::Switcher;
pub use words::Words;

pub const MAX_REJECTIONS: usize = 100;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Exceeded 100 rejections during generation.")]
    MaxRejectionsExceeded,

    #[error("Invalid hint \"{hint}\" for generator with ID \"{id}\".")]
    InvalidHint { hint: String, id: String },

    #[error("Generator with ID \"{id}\" cannot produce output matching the given hints.")]
    Overconstrained { id: String },
}

pub type Result<T> = StdResult<T, Error>;

pub trait Generator {
    fn generate(&self, rng: &mut dyn Rng, hints: &HashMap<&str, &str>) -> Result<Vec<String>>;
    fn analyze(&self, verbose: bool, indent: usize);
    fn id(&self) -> Option<&str> {
        None
    }
}
