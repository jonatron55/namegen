mod boolean_constraint;
mod number_constraint;
mod text_constraint;

use std::collections::HashSet;

use libnamegen::config::GeneratorConfig;

pub use boolean_constraint::BooleanConstraint;
pub use number_constraint::NumberConstraint;
pub use text_constraint::TextConstraint;

pub trait Constraints {
    fn constraints(&self) -> Vec<Constraint>;
}

#[derive(PartialEq, Clone)]
pub enum Constraint {
    Number { id: String, min: usize, max: usize },
    Text { id: String },
    Boolean { id: String },
}

impl Constraints for GeneratorConfig {
    fn constraints(&self) -> Vec<Constraint> {
        match self {
            GeneratorConfig::Description { subpart, .. } => subpart.constraints(),
            GeneratorConfig::Capitalizer { subpart, .. } => subpart.constraints(),
            GeneratorConfig::Joiner { subparts, .. } => {
                let mut views: Vec<_> = subparts.iter().flat_map(|s| s.constraints()).collect();
                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Markov { id: Some(id), .. } => vec![Constraint::Text { id: id.clone() }],
            GeneratorConfig::Matcher {
                base, cases, default, ..
            } => {
                let mut views = base.constraints();

                for (_, config) in cases {
                    views.extend(config.constraints());
                }

                if let Some(default) = default {
                    views.extend(default.constraints());
                }

                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Numberer {
                id: Some(id), min, max, ..
            } => vec![Constraint::Number {
                id: id.clone(),
                min: *min,
                max: *max,
            }],
            GeneratorConfig::Optional { id, generator, .. } => {
                let mut views = generator.constraints();
                if let Some(id) = id {
                    views.push(Constraint::Boolean { id: id.clone() });
                }
                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Repeater {
                id,
                generator,
                min,
                max,
            } => {
                let mut views = generator.constraints();
                if let Some(id) = id {
                    views.push(Constraint::Number {
                        id: id.clone(),
                        min: *min,
                        max: *max,
                    });
                }
                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Switcher { id, subparts, .. } => {
                let mut views: Vec<_> = subparts.iter().flat_map(|s| s.constraints()).collect();
                let mut seen = HashSet::new();
                if let Some(id) = id {
                    views.push(Constraint::Number {
                        id: id.clone(),
                        min: 0,
                        max: subparts.len() - 1,
                    });
                }
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Words { id: Some(id), .. } => vec![Constraint::Text { id: id.clone() }],
            _ => vec![],
        }
    }
}

impl Constraint {
    pub fn id(&self) -> &String {
        match self {
            Constraint::Number { id, .. } => id,
            Constraint::Text { id, .. } => id,
            Constraint::Boolean { id, .. } => id,
        }
    }
}
