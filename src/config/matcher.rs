use std::io::Write;

use regex::Regex;
use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, Matcher},
};

pub struct MatcherConfig {
    base: Box<dyn GeneratorConfig>,
    cases: Vec<(Regex, Box<dyn GeneratorConfig>)>,
    default: Option<Box<dyn GeneratorConfig>>,
}

impl MatcherConfig {
    pub fn new(
        base: Box<dyn GeneratorConfig>,
        cases: Vec<(Regex, Box<dyn GeneratorConfig>)>,
        default: Option<Box<dyn GeneratorConfig>>,
    ) -> Self {
        Self { base, cases, default }
    }
}

impl GeneratorConfig for MatcherConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Matcher::new(
            self.base.into_generator(),
            self.cases
                .into_iter()
                .map(|(regex, config)| (regex, config.into_generator()))
                .collect(),
            self.default.map(|config| config.into_generator()),
        ))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        writer.write(XmlEvent::start_element("Match"))?;
        self.base.write_xml(writer, indent + 2)?;

        for (regex, config) in self.cases {
            writer.write(XmlEvent::start_element("Case").attr("expr", &regex.as_str()))?;
            config.write_xml(writer, indent + 4)?;
            writer.write(XmlEvent::end_element())?;
        }

        if let Some(default) = self.default {
            writer.write(XmlEvent::start_element("Default"))?;
            default.write_xml(writer, indent + 2)?;
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
