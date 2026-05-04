use std::collections::HashMap;

use crate::{
    generator::{Error, Generator, Result},
    styles::{ELEM, ID, PROP},
};

pub struct Capitalizer {
    id: Option<String>,
    subpart: Box<dyn Generator>,
    mode: CapitalizerMode,
}

#[derive(Clone, Copy, Debug)]
pub enum CapitalizerMode {
    AllLower,
    FirstUpper,
    AllUpper,
}

impl Capitalizer {
    pub fn new(id: Option<String>, subpart: Box<dyn Generator>, mode: CapitalizerMode) -> Self {
        Self { id, subpart, mode }
    }

    fn capitalize(&self, s: String) -> String {
        match self.mode {
            CapitalizerMode::AllLower => s.to_lowercase(),
            CapitalizerMode::FirstUpper => {
                let mut chars = s.chars();

                if let Some(first) = chars.next() {
                    let mut s = String::with_capacity(s.len());
                    s.extend(first.to_uppercase());
                    s.extend(chars.flat_map(|ch| ch.to_lowercase()));
                    s
                } else {
                    String::new()
                }
            }
            CapitalizerMode::AllUpper => s.to_uppercase(),
        }
    }
}

impl Generator for Capitalizer {
    fn generate(&self, rand: &mut dyn rand::Rng, constraints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        if let Some(id) = self.id.as_deref()
            && let Some(constraint) = constraints.get(id)
        {
            return Err(Error::InvalidHint {
                constraint: constraint.to_string(),
                id: id.to_string(),
            });
        }

        self.subpart
            .generate(rand, constraints)
            .and_then(|vec| Ok(vec.into_iter().map(|s| self.capitalize(s)).collect()))
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!(
            "{}{ELEM}Capitalizer{ELEM:#} {ID}{}{ID:#}",
            indent_str,
            self.id.as_deref().unwrap_or("unnamed")
        );
        println!("{} {PROP}Mode: {PROP:#}{:?}", indent_str, self.mode);
        self.subpart.analyze(verbose, indent + 2);
    }
}
