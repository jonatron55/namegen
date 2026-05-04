mod into_generator;
mod parser;
mod write_xml;

use std::io::{self, Error as IoError, Read};

use regex::Regex;
use thiserror::Error as ThisError;
use xml::{ParserConfig as XmlParserConfig, writer::Error as XmlWriteError};

pub use into_generator::IntoGenerator;
pub use parser::Error as ParseError;
use parser::from_xml;
pub use write_xml::WriteXml;

use crate::generator::{CapitalizerMode, NumberStyle, Tokenizer};

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

pub enum GeneratorConfig {
    Capitalizer {
        id: Option<String>,
        subpart: Box<GeneratorConfig>,
        mode: CapitalizerMode,
    },
    Joiner {
        id: Option<String>,
        subparts: Vec<Box<GeneratorConfig>>,
        sep: String,
        reject: Vec<String>,
    },
    Literal {
        id: Option<String>,
        text: String,
    },
    Markov {
        id: Option<String>,
        data: Vec<String>,
        target_len: Option<usize>,
        cutoff_len: Option<usize>,
        reject: Vec<String>,
        uniform: bool,
        reject_training: bool,
        tokenizer: Tokenizer,
    },
    Matcher {
        id: Option<String>,
        base: Box<GeneratorConfig>,
        cases: Vec<(Regex, Box<GeneratorConfig>)>,
        default: Option<Box<GeneratorConfig>>,
    },
    Numberer {
        id: Option<String>,
        min: usize,
        max: usize,
        style: NumberStyle,
    },
    Optional {
        id: Option<String>,
        generator: Box<GeneratorConfig>,
        probability: f64,
    },
    Repeater {
        id: Option<String>,
        generator: Box<GeneratorConfig>,
        min: usize,
        max: usize,
    },
    Switcher {
        id: Option<String>,
        subparts: Vec<Box<GeneratorConfig>>,
    },
    Words {
        id: Option<String>,
        words: Vec<String>,
    },
}

impl GeneratorConfig {
    pub fn read(reader: impl Read, src_type: ConfigSourceType) -> Result<Box<GeneratorConfig>, ParseError> {
        match src_type {
            ConfigSourceType::PlainText => {
                let text = io::read_to_string(reader)?;
                let mut data: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
                data.dedup();

                Ok(Box::new(GeneratorConfig::Markov {
                    id: Some("name".to_string()),
                    data,
                    target_len: None,
                    cutoff_len: None,
                    reject: vec![],
                    uniform: false,
                    reject_training: false,
                    tokenizer: Tokenizer::default_ssp(),
                }))
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
}
