use crate::generator::{Generator, Result};

pub struct Capitalizer {
    subpart: Box<dyn Generator>,
    mode: CapitalizerMode,
}

#[derive(Clone, Debug)]
pub enum CapitalizerMode {
    AllLower,
    FirstUpper,
    AllUpper,
}

impl Capitalizer {
    pub fn new(subpart: Box<dyn Generator>, mode: CapitalizerMode) -> Self {
        Self { subpart, mode }
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
    fn generate(&self, rand: &mut dyn rand::Rng) -> Result<Vec<String>> {
        self.subpart
            .generate(rand)
            .and_then(|vec| Ok(vec.into_iter().map(|s| self.capitalize(s)).collect()))
    }

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("{}Capitalizer: mode={:?}", indent_str, self.mode);
        print!("{} Subpart: ", indent_str);
        self.subpart.print_analysis(indent + 2);
    }
}
