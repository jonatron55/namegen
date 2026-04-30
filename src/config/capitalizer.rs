use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Capitalizer, CapitalizerMode, Generator},
};

pub struct CapitalizerConfig {
    subpart: Box<dyn GeneratorConfig>,
    mode: CapitalizerMode,
}

impl CapitalizerConfig {
    pub fn new(subpart: Box<dyn GeneratorConfig>, mode: CapitalizerMode) -> Self {
        Self { subpart, mode }
    }
}

impl GeneratorConfig for CapitalizerConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        let subgen = self.subpart.into_generator();
        Box::new(Capitalizer::new(subgen, self.mode))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        if matches!(self.mode, CapitalizerMode::FirstUpper) {
            writer.write(XmlEvent::start_element("Capitalize"))?;
        } else {
            writer.write(XmlEvent::start_element("Capitalize").attr("mode", &format!("{:?}", self.mode)))?;
        }
        self.subpart.write_xml(writer, indent + 2)?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
