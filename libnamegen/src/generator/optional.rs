use rand::{Rng, RngExt};

use crate::generator::{Constraints, Error, Generator, Result};

pub struct Optional {
    id: Option<String>,
    generator: Box<dyn Generator>,
    probability: f64,
}

impl Optional {
    pub fn new(id: Option<String>, generator: Box<dyn Generator>, probability: f64) -> Self {
        Self {
            id,
            generator,
            probability,
        }
    }
}

impl Generator for Optional {
    fn generate(&self, rand: &mut dyn Rng, constraints: &dyn Constraints) -> Result<Vec<String>> {
        let roll = if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            match constraint.parse::<bool>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(Error::InvalidHint {
                        constraint: constraint.to_string(),
                        id: id.to_string(),
                    });
                }
            }
        } else {
            rand.random_bool(self.probability)
        };

        if roll {
            self.generator.generate(rand, constraints)
        } else {
            Ok(vec![])
        }
    }
}
