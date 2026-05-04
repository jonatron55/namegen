use std::collections::HashMap;

use rand::{Rng, RngExt};

pub struct Numberer {
    id: Option<String>,
    min: usize,
    max: usize,
    style: NumberStyle,
}

use crate::{
    generator::{Error, Generator, Result},
    styles::{ELEM, ID, PROP},
};

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
    pub fn new(id: Option<String>, min: usize, max: usize, style: NumberStyle) -> Self {
        Self { id, min, max, style }
    }
}

impl Generator for Numberer {
    fn generate(&self, rand: &mut dyn Rng, constraints: &HashMap<&str, &str>) -> Result<Vec<String>> {
        let num = if let Some(id) = self.id.as_deref()
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

        Ok(vec![self.style.format(num)])
    }

    fn analyze(&self, _verbose: bool, indent: usize) {
        let indent_str = " ".repeat(indent);
        println!(
            "{}{ELEM}Number generator{ELEM:#} {ID}{}{ID:#}:",
            indent_str,
            self.id.as_deref().unwrap_or("unnamed")
        );
        println!("{} {PROP}Min: {PROP:#}{}", indent_str, self.min);
        println!("{} {PROP}Max: {PROP:#}{}", indent_str, self.max);
        println!("{} {PROP}Style: {PROP:#}{:?}", indent_str, self.style);
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
