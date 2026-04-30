use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, Repeater},
};

pub struct RepeaterConfig {
    generator: Box<dyn GeneratorConfig>,
    min: usize,
    max: usize,
}

impl RepeaterConfig {
    pub fn new(generator: Box<dyn GeneratorConfig>, min: usize, max: usize) -> Self {
        Self { generator, min, max }
    }
}

impl GeneratorConfig for RepeaterConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Repeater::new(self.generator.into_generator(), self.min, self.max))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        writer.write(
            XmlEvent::start_element("Repeat")
                .attr("min", &self.min.to_string())
                .attr("max", &self.max.to_string()),
        )?;
        self.generator.write_xml(writer, indent + 2)?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
