use crate::generator::Generator;

use rand::Rng;
use rand::seq::IndexedRandom;

pub struct Switcher {
    subparts: Vec<Box<dyn Generator>>,
}

impl Switcher {
    pub fn new(subparts: Vec<Box<dyn Generator>>) -> Self {
        Self { subparts }
    }
}

impl Generator for Switcher {
    fn generate(&self, rand: &mut dyn Rng) -> Vec<String> {
        self.subparts
            .choose(rand)
            .map(|part| part.generate(rand))
            .unwrap_or_else(Vec::new)
    }
}
