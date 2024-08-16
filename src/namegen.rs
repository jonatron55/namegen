pub mod config;

use std::boxed::Box;
use std::mem::take;

use config::{Config as NameGenConfig, PartConfig};
use itertools::Itertools;
use rand::rngs::ThreadRng;

use crate::{concatter::Concatter, markov::MarkovGen};

pub struct NameGen {
    parts: Vec<Part>,
}

struct Part {
    generator: Box<dyn PartGen>,
    reject: Vec<String>,
}

pub trait PartGen {
    fn generate(&self, rng: &mut ThreadRng) -> String;
}

impl NameGen {
    pub fn from_config(mut config: NameGenConfig) -> NameGen {
        NameGen {
            parts: config
                .parts
                .iter_mut()
                .map(|part| match part {
                    PartConfig::MarkovPart {
                        training_data,
                        ref mut reject,
                    } => {
                        Part {
                            generator: Box::new(MarkovGen::train(training_data)),
                            reject: take(reject),
                        }
                    }
                    PartConfig::ConcatPart {
                        ref mut subparts,
                        ref mut reject } => {
                        Part {
                            generator: Box::new(Concatter::new(take(subparts))),
                            reject: take(reject),
                        }
                    }
                })
                .collect(),
        }
    }

    pub fn generate(&self, rng: &mut ThreadRng) -> String {
        self.parts
            .iter()
            .map(|part|
                loop {
                    let g = part.generator.generate(rng);
                    if !part.reject.contains(&g) {
                        break g;
                    }
                })
            .intersperse(" ".to_string())
            .collect()
    }
}
