use std::io::Write;

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{Generator, NumberStyle, Numberer},
};

use xml::{EventWriter as XmlWriter, writer::XmlEvent};

pub struct NumbererConfig {
    min: usize,
    max: usize,
    style: NumberStyle,
}

impl NumbererConfig {
    pub fn new(min: usize, max: usize) -> Self {
        Self {
            min,
            max,
            style: NumberStyle::Decimal,
        }
    }

    pub fn with_style(mut self, style: NumberStyle) -> Self {
        self.style = style;
        self
    }
}

impl GeneratorConfig for NumbererConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        Box::new(Numberer::new(self.min, self.max, self.style))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        _indent: usize,
    ) -> Result<(), WriteError> {
        writer.write(
            XmlEvent::start_element("Number")
                .attr("min", &self.min.to_string())
                .attr("max", &self.max.to_string())
                .attr("style", &format!("{:?}", self.style)),
        )?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
