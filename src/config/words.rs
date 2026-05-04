use std::io::Write;

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError, write_indented_lines},
    generator::{Generator, Words},
};

pub struct WordsConfig {
    id: Option<String>,
    words: Vec<String>,
}

impl WordsConfig {
    pub fn new(id: Option<String>, mut words: Vec<String>) -> Self {
        words.dedup();
        Self { id, words }
    }
}

impl GeneratorConfig for WordsConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Words::new(self.id, self.words))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        let mut ev = XmlEvent::start_element("Words");
        if let Some(id) = &self.id {
            ev = ev.attr("id", id);
        }

        writer.write(ev)?;
        write_indented_lines(self.words, indent + 2, writer)?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
