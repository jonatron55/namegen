use std::collections::HashMap;

use rand::Rng;

use crate::generator::{Error, Generator, Result};

pub struct Literal {
    id: Option<String>,
    text: String,
}

impl Literal {
    pub fn new(id: Option<String>, text: String) -> Self {
        Self { id, text }
    }
}

impl Generator for Literal {
    fn generate(&self, _rand: &mut dyn Rng, constraints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            return Err(Error::InvalidHint {
                constraint: constraint.to_string(),
                id: id.to_string(),
            });
        }

        Ok(vec![self.text.clone()])
    }
}
