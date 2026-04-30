mod capitalizer;
mod concatter;
mod markov;
mod numberer;
mod optional;
mod parser;
mod repeater;
mod switcher;

use std::io::{self, Error as IoError, Read, Write};

use thiserror::Error as ThisError;
use xml::{
    EventWriter as XmlWriter, ParserConfig as XmlParserConfig,
    writer::{Error as XmlWriteError, XmlEvent},
};

pub use parser::Error as ParseError;
use parser::from_xml;

use crate::{
    config::markov::MarkovConfig,
    generator::{Generator, Tokenizer},
};

pub enum ConfigSourceType {
    PlainText,
    Xml,
}

#[derive(ThisError, Debug)]
pub enum WriteError {
    #[error("{0}")]
    Io(#[from] IoError),

    #[error("{0}")]
    Xml(#[from] XmlWriteError),
}

pub trait GeneratorConfig {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator>;
    fn write_xml(self: Box<Self>, writer: &mut XmlWriter<&mut Box<dyn Write>>, indent: usize)
    -> Result<(), WriteError>;
}

pub fn read(reader: impl Read, src_type: ConfigSourceType) -> Result<Box<dyn GeneratorConfig>, ParseError> {
    match src_type {
        ConfigSourceType::PlainText => {
            let text = io::read_to_string(reader)?;
            let data = text.split_whitespace().map(|s| s.to_string()).collect();
            Ok(Box::new(MarkovConfig::new(
                data,
                None,
                None,
                vec![],
                false,
                false,
                Tokenizer::default_ssp(),
            )))
        }
        ConfigSourceType::Xml => {
            let mut xml = XmlParserConfig::new()
                .trim_whitespace(true)
                .whitespace_to_characters(true)
                .ignore_comments(true)
                .create_reader(reader);
            from_xml(&mut xml)
        }
    }
}

impl GeneratorConfig for Vec<String> {
    fn into_generator(self: Box<Self>) -> Box<dyn Generator> {
        self
    }

    fn write_xml(
        mut self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        self.sort_unstable();

        writer.write(XmlEvent::start_element("Words"))?;
        write_indented_lines(*self, indent + 2, writer)?;
        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}

const WRAP_WIDTH: usize = 80;

fn write_indented_lines(
    mut words: Vec<String>,
    indent: usize,
    writer: &mut XmlWriter<&mut Box<dyn Write>>,
) -> Result<(), WriteError> {
    let indent_str = " ".repeat(indent);
    writer.write(XmlEvent::characters("\n"))?;
    writer.write(XmlEvent::characters(&indent_str))?;

    words.sort_unstable();
    let mut line = String::with_capacity(WRAP_WIDTH);

    for word in words.iter() {
        if line.len() > 0 && line.len() + word.len() + 1 > WRAP_WIDTH {
            writer.write(XmlEvent::characters(&line))?;
            writer.write(XmlEvent::characters("\n"))?;
            writer.write(XmlEvent::characters(&indent_str))?;

            line.clear();
        }

        if !line.is_empty() {
            line.push(' ');
        }

        line.push_str(&word);
    }

    if !line.is_empty() {
        writer.write(XmlEvent::characters(&line))?;
        writer.write(XmlEvent::characters("\n"))?;
        let indent_str = " ".repeat(indent.saturating_sub(2));
        writer.write(XmlEvent::characters(&indent_str))?;
    }

    Ok(())
}
