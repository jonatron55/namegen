use rand::Rng;
use regex::Regex;

use crate::{
    generator::{Generator, Result},
    styles::ELEM,
};

pub struct Matcher {
    pub base: Box<dyn Generator>,
    pub cases: Vec<(Regex, Box<dyn Generator>)>,
    pub default: Option<Box<dyn Generator>>,
}

impl Matcher {
    pub fn new(
        base: Box<dyn Generator>,
        cases: Vec<(Regex, Box<dyn Generator>)>,
        default: Option<Box<dyn Generator>>,
    ) -> Self {
        Self { base, cases, default }
    }
}

impl Generator for Matcher {
    fn generate(&self, rng: &mut dyn Rng) -> Result<Vec<String>> {
        let mut base_outputs = self.base.generate(rng)?;

        if let Some(output) = base_outputs.last() {
            for (regex, generator) in &self.cases {
                if regex.is_match(output) {
                    base_outputs.append(&mut generator.generate(rng)?);
                    return Ok(base_outputs);
                }
            }
        }

        if let Some(default_gen) = &self.default {
            base_outputs.append(&mut default_gen.generate(rng)?);
        }

        Ok(base_outputs)
    }

    fn analyze(&self, verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}{ELEM}Matcher:{ELEM:#}", indent_str);

        self.base.analyze(verbose, indent + 2);

        for (regex, generator) in &self.cases {
            println!("{}{ELEM}Case:{ELEM:#} Regex: {:?}", indent_str, regex);
            generator.analyze(verbose, indent + 4);
        }

        if let Some(default_gen) = &self.default {
            println!("{}{ELEM}Default:{ELEM:#}", indent_str);
            default_gen.analyze(verbose, indent + 2);
        }
    }
}
