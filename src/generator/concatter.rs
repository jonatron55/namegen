use itertools::Itertools;
use rand::Rng;

use crate::generator::Generator;

pub struct Concatter {
    subparts: Vec<Box<dyn Generator>>,
    joiner: String,
    reject: Vec<String>,
}

impl Concatter {
    pub fn new(subparts: Vec<Box<dyn Generator>>, reject: Vec<String>) -> Self {
        Self {
            subparts,
            joiner: "".to_string(),
            reject,
        }
    }

    pub fn with_joiner(mut self, joiner: String) -> Self {
        self.joiner = joiner;
        self
    }
}

impl Generator for Concatter {
    #[allow(unstable_name_collisions)] // `intersperse` may at some point be incorporated into the standard library, but for now we need to use the one from itertools
    fn generate(&self, rand: &mut dyn Rng) -> Vec<String> {
        loop {
            let name: Vec<_> = self.subparts.iter().flat_map(|part| part.generate(rand)).collect();

            // Reject duplicates
            if name.iter().tuple_windows().any(|(a, b)| a == b) {
                continue;
            }

            let name = name.join(&self.joiner);
            if !self.reject.contains(&name) {
                return vec![name];
            }
        }
    }
}
