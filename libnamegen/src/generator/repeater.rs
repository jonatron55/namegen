use std::collections::HashMap;

use rand::{Rng, RngExt};

use crate::generator::{Error, Generator, Result};

pub struct Repeater {
    id: Option<String>,
    generator: Box<dyn Generator>,
    min: usize,
    max: usize,
}

impl Repeater {
    pub fn new(id: Option<String>, generator: Box<dyn Generator>, min: usize, max: usize) -> Self {
        Self {
            id,
            generator,
            min,
            max,
        }
    }
}

impl Generator for Repeater {
    fn generate(&self, rand: &mut dyn Rng, constraints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        let count = if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            match constraint.parse::<usize>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(Error::InvalidHint {
                        constraint: constraint.to_string(),
                        id: id.to_string(),
                    });
                }
            }
        } else {
            rand.random_range(self.min..=self.max)
        };

        (0..count).try_fold(Vec::new(), |mut acc, _| {
            self.generator.generate(rand, constraints).map(|mut v| {
                acc.append(&mut v);
                acc
            })
        })
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}
