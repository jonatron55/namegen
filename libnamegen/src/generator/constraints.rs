use std::collections::HashMap;

pub trait Constraints {
    fn get(&self, id: &str) -> Option<&str>;
}

impl Constraints for HashMap<String, String> {
    fn get(&self, id: &str) -> Option<&str> {
        self.get(id).map(|s| s.as_str())
    }
}

impl Constraints for HashMap<&str, &str> {
    fn get(&self, id: &str) -> Option<&str> {
        self.get(id).map(|s| *s)
    }
}
