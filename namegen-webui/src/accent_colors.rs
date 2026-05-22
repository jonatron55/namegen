use rand::{Rng, RngExt};

#[derive(Debug)]
pub struct AccentColors<R: Rng> {
    current: usize,
    last: usize,
    rng: R,
}

impl<R: Rng> AccentColors<R> {
    pub fn new(rng: R) -> Self {
        Self {
            current: 1,
            last: 0,
            rng,
        }
    }

    pub fn next(&mut self) -> usize {
        loop {
            let next = self.rng.random_range(1..6);
            if next != self.current {
                self.last = self.current;
                self.current = next;
                return self.current;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColoredString {
    pub text: String,
    pub color: usize,
}

impl ColoredString {
    pub fn class(&self) -> String {
        format!("color-{}", self.color)
    }
}

pub trait WithAccentColor {
    fn with_accent_color(self, color: usize) -> ColoredString;
}

impl WithAccentColor for String {
    fn with_accent_color(self, color: usize) -> ColoredString {
        ColoredString { text: self, color }
    }
}
