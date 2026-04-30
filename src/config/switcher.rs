use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, Switcher},
};

pub struct SwitcherConfig {
    subparts: Vec<Box<dyn GeneratorConfig>>,
}

impl SwitcherConfig {
    pub fn new(subparts: Vec<Box<dyn GeneratorConfig>>) -> Self {
        Self { subparts }
    }
}

impl GeneratorConfig for SwitcherConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Switcher::new(
            self.subparts.into_iter().map(|config| config.into_generator()).collect(),
        ))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        writer.write(XmlEvent::start_element("Switch"))?;
        for subpart in self.subparts {
            subpart.write_xml(writer, indent + 2)?;
        }
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
