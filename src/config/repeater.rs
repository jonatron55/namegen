use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, Repeater},
};

pub struct RepeaterConfig {
    id: Option<String>,
    generator: Box<dyn GeneratorConfig>,
    min: usize,
    max: usize,
}

impl RepeaterConfig {
    pub fn new(id: Option<String>, generator: Box<dyn GeneratorConfig>, min: usize, max: usize) -> Self {
        Self {
            id,
            generator,
            min,
            max,
        }
    }
}

impl GeneratorConfig for RepeaterConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Repeater::new(
            self.id,
            self.generator.into_generator(),
            self.min,
            self.max,
        ))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        let mut ev = XmlEvent::start_element("Repeat");
        if let Some(id) = &self.id {
            ev = ev.attr("id", id);
        }

        writer.write(ev.attr("min", &self.min.to_string()).attr("max", &self.max.to_string()))?;
        self.generator.write_xml(writer, indent + 2)?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
