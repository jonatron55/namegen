use crate::{
    config::GeneratorConfig,
    generator::{
        Capitalizer, Generator, Joiner, Literal, Markov, Matcher, Numberer, Optional, Repeater, Switcher, Words,
    },
};

pub trait IntoGenerator {
    fn into_generator(self) -> Box<dyn Generator>;
}

impl IntoGenerator for GeneratorConfig {
    fn into_generator(self) -> Box<dyn Generator> {
        match self {
            GeneratorConfig::Description { subpart, .. } => subpart.into_generator(),
            GeneratorConfig::Capitalizer { id, subpart, mode } => {
                (Box::new(Capitalizer::new(id, subpart.into_generator(), mode))) as Box<dyn Generator>
            }
            GeneratorConfig::Joiner {
                id,
                subparts,
                sep,
                reject,
            } => Box::new(Joiner::new(
                id,
                subparts.into_iter().map(IntoGenerator::into_generator).collect(),
                sep,
                reject,
            )) as Box<dyn Generator>,
            GeneratorConfig::Literal { id, text } => Box::new(Literal::new(id, text)) as Box<dyn Generator>,
            GeneratorConfig::Markov {
                id,
                data,
                target_len,
                cutoff_len,
                mut reject,
                uniform,
                reject_training,
                tokenizer,
            } => {
                if reject_training {
                    reject.extend_from_slice(&data);
                    reject.dedup();
                }

                Box::new(Markov::train(
                    id, data, target_len, cutoff_len, reject, tokenizer, uniform,
                )) as Box<dyn Generator>
            }
            GeneratorConfig::Matcher {
                id,
                base,
                cases,
                default,
            } => Box::new(Matcher::new(
                id,
                base.into_generator(),
                cases
                    .into_iter()
                    .map(|(regex, config)| (regex, config.into_generator()))
                    .collect(),
                default.map(IntoGenerator::into_generator),
            )) as Box<dyn Generator>,
            GeneratorConfig::Numberer { id, min, max, style } => {
                Box::new(Numberer::new(id, min, max, style)) as Box<dyn Generator>
            }
            GeneratorConfig::Optional {
                id,
                generator,
                probability,
            } => Box::new(Optional::new(id, generator.into_generator(), probability)) as Box<dyn Generator>,
            GeneratorConfig::Repeater {
                id,
                generator,
                min,
                max,
            } => Box::new(Repeater::new(id, generator.into_generator(), min, max)) as Box<dyn Generator>,
            GeneratorConfig::Switcher { id, subparts } => Box::new(Switcher::new(
                id,
                subparts.into_iter().map(IntoGenerator::into_generator).collect(),
            )) as Box<dyn Generator>,
            GeneratorConfig::Words { id, words } => Box::new(Words::new(id, words)) as Box<dyn Generator>,
        }
    }
}

impl IntoGenerator for Box<GeneratorConfig> {
    fn into_generator(self) -> Box<dyn Generator> {
        (*self).into_generator()
    }
}
