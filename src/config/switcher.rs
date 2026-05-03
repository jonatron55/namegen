use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, Switcher},
};

pub struct SwitcherConfig {
    id: Option<String>,
    subparts: Vec<Box<dyn GeneratorConfig>>,
}

impl SwitcherConfig {
    pub fn new(id: Option<String>, subparts: Vec<Box<dyn GeneratorConfig>>) -> Self {
        Self { id, subparts }
    }
}

impl GeneratorConfig for SwitcherConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Switcher::new(
            self.id,
            self.subparts.into_iter().map(|config| config.into_generator()).collect(),
        ))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        let mut ev = XmlEvent::start_element("Switch");
        if let Some(id) = &self.id {
            ev = ev.attr("id", id);
        }

        writer.write(ev)?;
        for subpart in self.subparts {
            subpart.write_xml(writer, indent + 2)?;
        }
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
