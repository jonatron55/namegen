use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, Optional},
};

pub struct OptionalConfig {
    generator: Box<dyn GeneratorConfig>,
    probability: f64,
}

impl OptionalConfig {
    pub fn new(generator: Box<dyn GeneratorConfig>, probability: f64) -> Self {
        Self { generator, probability }
    }
}

impl GeneratorConfig for OptionalConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Optional::new(self.generator.into_generator(), self.probability))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        writer.write(XmlEvent::start_element("Option").attr("probability", &format!("{}", self.probability)))?;
        self.generator.write_xml(writer, indent + 2)?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
