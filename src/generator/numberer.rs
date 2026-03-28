use crate::generator::Generator;

use rand::{Rng, RngExt};

pub struct Numberer {
    min: usize,
    max: usize,
    style: NumberStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberStyle {
    Decimal,
    HexadecimalUpper,
    HexadecimalLower,
    Octal,
    Binary,
    RomanUpper,
    RomanLower,
}

impl Numberer {
    pub fn new(min: usize, max: usize) -> Self {
        Self {
            min,
            max,
            style: NumberStyle::Decimal,
        }
    }

    pub fn with_style(mut self, style: NumberStyle) -> Self {
        self.style = style;
        self
    }
}

impl Generator for Numberer {
    fn generate(&self, rand: &mut dyn Rng) -> Vec<String> {
        let num = rand.random_range(self.min..=self.max);
        vec![self.style.format(num)]
    }
}

const ROMAN_VALUES: [(usize, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

impl NumberStyle {
    fn format(&self, mut num: usize) -> String {
        match self {
            NumberStyle::Decimal => num.to_string(),
            NumberStyle::HexadecimalUpper => format!("{:X}", num),
            NumberStyle::HexadecimalLower => format!("{:x}", num),
            NumberStyle::Octal => format!("{:o}", num),
            NumberStyle::Binary => format!("{:b}", num),
            NumberStyle::RomanLower => NumberStyle::RomanLower.format(num).to_lowercase(),
            NumberStyle::RomanUpper => {
                let mut result = String::new();

                for &(value, symbol) in &ROMAN_VALUES {
                    while num >= value {
                        result.push_str(symbol);
                        num -= value;
                    }
                }

                result
            }
        }
    }
}
