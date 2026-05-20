use rand::Rng;
use rand::seq::IndexedRandom;

use crate::generator::{Constraints, Error, Generator, Result};

pub struct Switcher {
    id: Option<String>,
    subparts: Vec<Box<dyn Generator>>,
}

impl Switcher {
    pub fn new(id: Option<String>, subparts: Vec<Box<dyn Generator>>) -> Self {
        Self { id, subparts }
    }
}

impl Generator for Switcher {
    fn generate(&self, rand: &mut dyn Rng, constraints: &dyn Constraints) -> Result<Vec<String>> {
        if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            let idx = constraint.parse::<usize>().map_err(|_| Error::InvalidHint {
                id: id.to_string(),
                constraint: constraint.to_string(),
            })?;
            if idx < self.subparts.len() {
                return self.subparts[idx].generate(rand, constraints);
            } else {
                return Err(Error::InvalidHint {
                    id: id.to_string(),
                    constraint: constraint.to_string(),
                });
            }
        }

        self.subparts
            .choose(rand)
            .map(|part| part.generate(rand, constraints))
            .unwrap_or_else(|| Ok(vec![]))
    }
}
