use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;

use crate::namegen::PartGen;

pub struct Concatter {
    subparts: Vec<String>,
}

impl Concatter {
    pub fn new(subparts: Vec<String>) -> Self {
        Self { subparts }
    }
}

impl PartGen for Concatter {
    fn generate(&self, _rand: &mut ThreadRng) -> String {
        self.subparts
            .iter()
            .map(|part| {
                part.split_whitespace()
                    .collect::<Vec<&str>>()
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .to_string()
            })
            .join("")
    }
}
