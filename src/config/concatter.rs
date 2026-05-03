use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError, write_indented_lines},
    generator::{Concatter, Generator},
};

pub struct ConcatterConfig {
    id: Option<String>,
    subparts: Vec<Box<dyn GeneratorConfig>>,
    joiner: String,
    reject: Vec<String>,
}

impl ConcatterConfig {
    pub fn new(id: Option<String>, subparts: Vec<Box<dyn GeneratorConfig>>, reject: Vec<String>) -> Self {
        Self {
            id,
            subparts,
            joiner: "".to_string(),
            reject,
        }
    }

    pub fn with_joiner(mut self, joiner: String) -> Self {
        self.joiner = joiner;
        self
    }
}

impl GeneratorConfig for ConcatterConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Concatter::new(
            self.id,
            self.subparts.into_iter().map(|config| config.into_generator()).collect(),
            self.joiner,
            self.reject,
        ))
    }

    fn write_xml(
        mut self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        let mut ev = XmlEvent::start_element("Concat");
        if let Some(id) = &self.id {
            ev = ev.attr("id", id);
        }

        if self.joiner.len() > 0 {
            writer.write(ev.attr("joiner", &self.joiner))?;
        } else {
            writer.write(ev)?;
        }

        for subpart in self.subparts {
            subpart.write_xml(writer, indent + 2)?;
        }

        if self.reject.len() > 0 {
            self.reject.sort_unstable();
            writer.write(XmlEvent::start_element("Reject"))?;
            write_indented_lines(self.reject, indent + 4, writer)?;
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
