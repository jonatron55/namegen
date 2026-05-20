use crate::{
    config::GeneratorConfig,
    generator::{
        Capitalizer, Generator, Joiner, Literal, Markov, Matcher, Numberer, Optional, Repeater, Switcher, Words,
    },
};

pub trait BuildGenerator {
    fn build_generator(&self) -> Box<dyn Generator>;
}

impl BuildGenerator for GeneratorConfig {
    fn build_generator(&self) -> Box<dyn Generator> {
        match self {
            GeneratorConfig::Description { subpart, .. } => subpart.build_generator(),
            GeneratorConfig::Capitalizer { id, subpart, mode } => {
                (Box::new(Capitalizer::new(id.clone(), subpart.build_generator(), *mode))) as Box<dyn Generator>
            }
            GeneratorConfig::Joiner {
                id,
                subparts,
                sep,
                reject,
            } => Box::new(Joiner::new(
                id.clone(),
                subparts.iter().map(BuildGenerator::build_generator).collect(),
                sep.clone(),
                reject.clone(),
            )) as Box<dyn Generator>,
            GeneratorConfig::Literal { id, text } => {
                Box::new(Literal::new(id.clone(), text.clone())) as Box<dyn Generator>
            }
            GeneratorConfig::Markov {
                id,
                data,
                target_len,
                cutoff_len,
                reject,
                uniform,
                reject_training,
                tokenizer,
            } => {
                let mut reject = reject.clone();

                if *reject_training {
                    reject.extend_from_slice(&data);
                    reject.dedup();
                }

                Box::new(Markov::train(
                    id.clone(),
                    &data,
                    *target_len,
                    *cutoff_len,
                    reject,
                    tokenizer.clone(),
                    *uniform,
                )) as Box<dyn Generator>
            }
            GeneratorConfig::Matcher {
                id,
                base,
                cases,
                default,
            } => Box::new(Matcher::new(
                id.clone(),
                base.build_generator(),
                cases
                    .iter()
                    .map(|(regex, config)| (regex.clone(), config.build_generator()))
                    .collect(),
                default.as_ref().map(BuildGenerator::build_generator),
            )) as Box<dyn Generator>,
            GeneratorConfig::Numberer { id, min, max, style } => {
                Box::new(Numberer::new(id.clone(), *min, *max, *style)) as Box<dyn Generator>
            }
            GeneratorConfig::Optional {
                id,
                generator,
                probability,
            } => Box::new(Optional::new(id.clone(), generator.build_generator(), *probability)) as Box<dyn Generator>,
            GeneratorConfig::Repeater {
                id,
                generator,
                min,
                max,
            } => Box::new(Repeater::new(id.clone(), generator.build_generator(), *min, *max)) as Box<dyn Generator>,
            GeneratorConfig::Switcher { id, subparts } => Box::new(Switcher::new(
                id.clone(),
                subparts.iter().map(BuildGenerator::build_generator).collect(),
            )) as Box<dyn Generator>,
            GeneratorConfig::Words { id, words } => {
                Box::new(Words::new(id.clone(), words.clone())) as Box<dyn Generator>
            }
        }
    }
}

impl BuildGenerator for Box<GeneratorConfig> {
    fn build_generator(&self) -> Box<dyn Generator> {
        (**self).build_generator()
    }
}
