mod capitalizer;
mod constraints;
mod joiner;
mod literal;
mod markov;
mod matcher;
mod numberer;
mod optional;
mod repeater;
mod switcher;
mod words;

use std::result::Result as StdResult;

use rand::Rng;
use thiserror::Error as ThisError;

pub use capitalizer::{Capitalizer, CapitalizerMode};
pub use constraints::Constraints;
pub use joiner::Joiner;
pub use literal::Literal;
pub use markov::{Markov, Tokenizer};
pub use matcher::Matcher;
pub use numberer::{NumberStyle, Numberer};
pub use optional::Optional;
pub use repeater::Repeater;
pub use switcher::Switcher;
pub use words::Words;

pub const MAX_REJECTIONS: usize = 1024;

#[derive(ThisError, Debug, Clone)]
pub enum Error {
    #[error("No output could be generated matching the given constraints after {MAX_REJECTIONS} attempts.")]
    MaxRejectionsExceeded,

    #[error("the constraint \"{constraint}\" is not valid for generator with ID \"{id}\".")]
    InvalidHint { constraint: String, id: String },

    #[error("The generator with ID \"{id}\" cannot produce output matching the given constraints.")]
    Overconstrained { id: String },
}

pub type Result<T> = StdResult<T, Error>;

pub trait Generator {
    fn generate(&self, rng: &mut dyn Rng, constraints: &dyn Constraints) -> Result<Vec<String>>;
    fn id(&self) -> Option<&str> {
        None
    }
}
