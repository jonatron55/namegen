use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError, write_indented_lines},
    generator::{Generator, Joiner},
};

pub struct JoinerConfig {
    id: Option<String>,
    subparts: Vec<Box<dyn GeneratorConfig>>,
    sep: String,
    reject: Vec<String>,
}

impl JoinerConfig {
    pub fn new(id: Option<String>, subparts: Vec<Box<dyn GeneratorConfig>>, reject: Vec<String>) -> Self {
        Self {
            id,
            subparts,
            sep: "".to_string(),
            reject,
        }
    }

    pub fn with_sep(mut self, sep: String) -> Self {
        self.sep = sep;
        self
    }
}

impl GeneratorConfig for JoinerConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Joiner::new(
            self.id,
            self.subparts.into_iter().map(|config| config.into_generator()).collect(),
            self.sep,
            self.reject,
        ))
    }

    fn write_xml(
        mut self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        let mut ev = XmlEvent::start_element("Join");
        if let Some(id) = &self.id {
            ev = ev.attr("id", id);
        }

        if self.sep.len() > 0 {
            writer.write(ev.attr("sep", &self.sep))?;
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
