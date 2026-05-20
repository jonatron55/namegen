use rand::Rng;
use regex::Regex;

use crate::generator::{Error, Generator, Result,Constraints};

pub struct Matcher {
    id: Option<String>,
    base: Box<dyn Generator>,
    cases: Vec<(Regex, Box<dyn Generator>)>,
    default: Option<Box<dyn Generator>>,
}

impl Matcher {
    pub fn new(
        id: Option<String>,
        base: Box<dyn Generator>,
        cases: Vec<(Regex, Box<dyn Generator>)>,
        default: Option<Box<dyn Generator>>,
    ) -> Self {
        Self {
            id,
            base,
            cases,
            default,
        }
    }
}

impl Generator for Matcher {
    fn generate(&self, rng: &mut dyn Rng, constraints: &dyn Constraints) -> Result<Vec<String>> {
        let mut base_outputs = self.base.generate(rng, constraints)?;

        if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            return Err(Error::InvalidHint {
                constraint: constraint.to_string(),
                id: id.to_string(),
            });
        }

        if let Some(output) = base_outputs.last() {
            for (regex, generator) in &self.cases {
                if regex.is_match(output) {
                    base_outputs.append(&mut generator.generate(rng, constraints)?);
                    return Ok(base_outputs);
                }
            }
        }

        if let Some(default_gen) = &self.default {
            base_outputs.append(&mut default_gen.generate(rng, constraints)?);
        }

        Ok(base_outputs)
    }
}
