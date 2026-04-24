use crate::generator::Generator;

use rand::{Rng, RngExt};

use crate::generator::Result;

pub struct Optional {
    generator: Box<dyn Generator>,
    propability: f64,
}

impl Optional {
    pub fn new(generator: Box<dyn Generator>, propability: f64) -> Self {
        Self { generator, propability }
    }
}

impl Generator for Optional {
    fn generate(&self, rand: &mut dyn Rng) -> Result<Vec<String>> {
        if rand.random_bool(self.propability) {
            self.generator.generate(rand)
        } else {
            Ok(vec![])
        }
    }

    fn print_analysis(&self, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!("Optional: propability {}", self.propability);
        print!("{} Subpart: ", indent_str);
        self.generator.print_analysis(indent + 2);
    }
}
