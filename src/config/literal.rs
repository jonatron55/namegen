use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, Literal},
};

pub struct LiteralConfig {
    id: Option<String>,
    text: String,
}

impl LiteralConfig {
    pub fn new(id: Option<String>, text: String) -> Self {
        Self { id, text }
    }
}

impl GeneratorConfig for LiteralConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Literal::new(self.id, self.text))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        _indent: usize,
    ) -> Result<(), WriteError> {
        let mut ev = XmlEvent::start_element("Literal");
        if let Some(id) = &self.id {
            ev = ev.attr("id", id);
        }

        writer.write(ev.attr("text", &self.text))?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
