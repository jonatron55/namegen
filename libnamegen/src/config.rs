mod build_generator;
mod elements;
mod into_generator;
mod parser;
mod write_xml;

use std::{
    collections::HashMap,
    io::{self, BufRead, Error as IoError, Read},
    path::Path,
};

use regex::Regex;
use thiserror::Error as ThisError;
use xml::{ParserConfig as XmlParserConfig, writer::Error as XmlWriteError};

pub use build_generator::BuildGenerator;
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

#[derive(Debug, Clone)]
pub enum GeneratorConfig {
    Description {
        display_name: String,
        description: String,
        arg_display_names: HashMap<String, String>,
        subpart: Box<GeneratorConfig>,
    },
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
    pub fn read(reader: impl Read, src_type: ConfigSourceType) -> Result<GeneratorConfig, ParseError> {
        match src_type {
            ConfigSourceType::PlainText => {
                let text = io::read_to_string(reader)?;
                let mut data: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
                data.dedup();

                Ok(GeneratorConfig::Description {
                    display_name: "Markov Generator".to_string(),
                    description: "Created from plain text input".to_string(),
                    arg_display_names: HashMap::from([("name".to_string(), "Name".to_string())]),
                    subpart: Box::new(GeneratorConfig::Markov {
                        id: Some("name".to_string()),
                        data,
                        target_len: None,
                        cutoff_len: None,
                        reject: vec![],
                        uniform: false,
                        reject_training: false,
                        tokenizer: Tokenizer::default_ssp(),
                    }),
                })
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

impl ConfigSourceType {
    /// Guess the source type of a configuration file based on its filename and contents.
    ///
    /// If the filename has a clear extension (`.xml` or `.txt`), that will be used. Otherwise, the function peeks at
    /// the start of the file for an opening `<` character (after skipping any whitespace or BOM) and guesses XML if one
    /// is found, or plain text otherwise.
    ///
    /// Arguments
    /// ---------
    ///
    /// - `filename`: The path to the configuration file. This is used to check the file extension.
    /// - `reader`: A buffered reader for the configuration file. The function will peek at the start of the file by
    ///   filling the buffer, but will not consume any bytes from the reader.
    pub fn guess(filename: &Path, reader: &mut impl BufRead) -> Result<ConfigSourceType, IoError> {
        match filename.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("xml") => return Ok(ConfigSourceType::Xml),
            Some(ext) if ext.eq_ignore_ascii_case("txt") => return Ok(ConfigSourceType::PlainText),
            _ => {}
        }

        let peek = reader.fill_buf()?;
        let mut prefix = &peek[..peek.len().min(16)];

        // Trim anything that looks like a BOM.
        prefix = match prefix {
            [0, 0, 0xFE, 0xFF, rest @ ..] | [0xFF, 0xFE, 0, 0, rest @ ..] => rest,
            [0xEF, 0xBB, 0xBF, rest @ ..] => rest,
            [0xFF, 0xFE, rest @ ..] | [0xFE, 0xFF, rest @ ..] => rest,
            _ => prefix,
        };

        // Skip any leading whitespace or zero bytes (UTF-16 and UTF-32 may have zero bytes in legitimate characters).
        while let Some((&byte, rest)) = prefix.split_first() {
            if byte.is_ascii_whitespace() || byte == 0 {
                prefix = rest;
            } else {
                break;
            }
        }

        // If the first non-whitespace, non-BOM character is `<`, assume it's XML.
        if prefix.starts_with(b"<") {
            Ok(ConfigSourceType::Xml)
        } else {
            Ok(ConfigSourceType::PlainText)
        }
    }
}
