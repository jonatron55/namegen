use std::{
    collections::{HashMap, HashSet},
    io::{Error as IoError, Read},
    result::Result as StdResult,
};

use lazy_static::lazy_static;
use regex::{Error as RegexError, Regex};
use thiserror::Error as ThisError;
use xml::{
    attribute::OwnedAttribute,
    common::{Position as XmlPosition, TextPosition},
    reader::{Error as XmlReadError, EventReader, XmlEvent},
};

use crate::{
    config::{
        GeneratorConfig, capitalizer::CapitalizerConfig, concatter::ConcatterConfig, markov::MarkovConfig,
        matcher::MatcherConfig, numberer::NumbererConfig, optional::OptionalConfig, repeater::RepeaterConfig,
        switcher::SwitcherConfig,
    },
    generator::{CapitalizerMode, NumberStyle, Tokenizer},
};

const ELEM_CAPITALIZE: &str = "Capitalize";
const ELEM_CASE: &str = "Case";
const ELEM_CONCAT: &str = "Concat";
const ELEM_DEFAULT: &str = "Default";
const ELEM_LITERAL: &str = "Literal";
const ELEM_MARKOV: &str = "Markov";
const ELEM_MATCH: &str = "Match";
const ELEM_NUMBER: &str = "Number";
const ELEM_OPTION: &str = "Option";
const ELEM_REPEAT: &str = "Repeat";
const ELEM_SWITCH: &str = "Switch";
const ELEM_WORDS: &str = "Words";

const ELEM_SPLIT_TOKENIZER: &str = "SplitTokenizer";
const ELEM_CHUNK_TOKENIZER: &str = "ChunkTokenizer";
const ELEM_SSP_TOKENIZER: &str = "SspTokenizer";
const ELEM_CLASS: &str = "Class";

lazy_static! {
    static ref VALID_PART_TYPES: HashSet<&'static str> = HashSet::from([
        ELEM_CAPITALIZE,
        ELEM_CONCAT,
        ELEM_LITERAL,
        ELEM_MARKOV,
        ELEM_MATCH,
        ELEM_NUMBER,
        ELEM_OPTION,
        ELEM_REPEAT,
        ELEM_SWITCH,
        ELEM_WORDS,
    ]);
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] IoError),

    #[error("{0}")]
    Xml(#[from] XmlReadError),

    #[error("Unexpected event {event:?}")]
    UnexpectedEvent { event: XmlEvent, position: TextPosition },

    #[error("Unexpected <{name}>")]
    UnexpectedElement { name: String, position: TextPosition },

    #[error("Unexpected end element: </{name}>")]
    UnexpectedEnd { name: String, position: TextPosition },

    #[error("Unexpected attribute: {name}")]
    UnexpectedAttribute { name: String, position: TextPosition },

    #[error("Invalid {attribute} value: \"{value}\"")]
    InvalidValue {
        attribute: String,
        value: String,
        position: TextPosition,
    },

    #[error("Invalid {attribute} value: {err}")]
    InvalidRegex {
        attribute: String,
        position: TextPosition,
        err: RegexError,
    },

    #[error("Missing attribute: \"{0}\"")]
    MissingAttribute(String),
}

pub type Result<T> = StdResult<T, Error>;

pub fn from_xml<R: Read>(reader: &mut EventReader<R>) -> Result<Box<dyn GeneratorConfig>> {
    let event = reader.next()?;

    match event {
        XmlEvent::StartDocument { .. } => {}
        other => {
            return Err(Error::UnexpectedEvent {
                event: other,
                position: reader.position(),
            });
        }
    }

    let event = reader.next()?;

    match event {
        XmlEvent::StartElement { name, .. } if name.local_name == "NameGen" => {}
        other => {
            return Err(Error::UnexpectedEvent {
                event: other,
                position: reader.position(),
            });
        }
    }

    let event = reader.next()?;
    let config = inner_from_xml(&event, reader)?;
    let event = reader.next()?;

    match event {
        XmlEvent::EndElement { name } => {
            if name.local_name == "NameGen" {
                Ok(config)
            } else {
                Err(Error::UnexpectedEnd {
                    name: name.local_name,
                    position: reader.position(),
                })
            }
        }
        other => Err(Error::UnexpectedEvent {
            event: other,
            position: reader.position(),
        }),
    }
}

fn inner_from_xml<R: Read>(event: &XmlEvent, reader: &mut EventReader<R>) -> Result<Box<dyn GeneratorConfig>> {
    match event {
        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_MARKOV => {
            let mut training_data = Vec::new();
            let mut reject = Vec::new();
            let mut reject_training = false;
            let mut uniform = false;
            let mut target_len = None;
            let mut cutoff_len = None;
            let mut tokenizer: Option<Tokenizer> = None;

            for attr in attributes {
                if attr.name.local_name == "target_len" {
                    target_len = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                        attribute: "target_len".to_string(),
                        value: attr.value.clone(),
                        position: reader.position(),
                    })?);
                } else if attr.name.local_name == "cutoff_len" {
                    cutoff_len = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                        attribute: "cutoff_len".to_string(),
                        value: attr.value.clone(),
                        position: reader.position(),
                    })?);
                } else if attr.name.local_name == "reject_training" {
                    reject_training = attr.value.parse().map_err(|_| Error::InvalidValue {
                        attribute: "reject_training".to_string(),
                        value: attr.value.clone(),
                        position: reader.position(),
                    })?;
                } else if attr.name.local_name == "uniform" {
                    uniform = attr.value.parse().map_err(|_| Error::InvalidValue {
                        attribute: "uniform".to_string(),
                        value: attr.value.clone(),
                        position: reader.position(),
                    })?;
                } else {
                    return Err(Error::UnexpectedAttribute {
                        name: attr.name.local_name.clone(),
                        position: reader.position(),
                    });
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement {
                        ref name,
                        ref attributes,
                        ..
                    } => match name.local_name.as_str() {
                        "Reject" => parse_reject(reader, &mut reject)?,
                        ELEM_SPLIT_TOKENIZER | ELEM_CHUNK_TOKENIZER | ELEM_SSP_TOKENIZER => {
                            if tokenizer.is_some() {
                                return Err(Error::UnexpectedElement {
                                    name: name.local_name.clone(),
                                    position: reader.position(),
                                });
                            }
                            tokenizer = Some(parse_tokenizer(reader, &name.local_name, attributes)?);
                        }
                        _ => {
                            return Err(Error::UnexpectedElement {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }
                    },
                    XmlEvent::Characters(data) => {
                        training_data.extend(data.split_whitespace().map(|s| s.to_string()));
                    }
                    XmlEvent::EndElement { name } => {
                        if name.local_name == ELEM_MARKOV {
                            let tokenizer = tokenizer.unwrap_or_default();
                            training_data.dedup();

                            return Ok(Box::new(MarkovConfig::new(
                                training_data,
                                target_len,
                                cutoff_len,
                                reject,
                                reject_training,
                                uniform,
                                tokenizer,
                            )));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name,
                                position: reader.position(),
                            });
                        }
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }
        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_MATCH => {
            let mut base = None;
            let mut cases = Vec::new();
            let mut default = None;

            for attr in attributes {
                return Err(Error::UnexpectedAttribute {
                    name: attr.name.local_name.clone(),
                    position: reader.position(),
                });
            }

            loop {
                let event = reader.next()?;
                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        base = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::StartElement {
                        ref name, attributes, ..
                    } if name.local_name == ELEM_CASE => {
                        let mut expr = None;

                        for attr in attributes {
                            if attr.name.local_name == "expr" {
                                expr = Some(Regex::new(&attr.value).map_err(|err| Error::InvalidRegex {
                                    attribute: "expr".to_string(),
                                    position: reader.position(),
                                    err,
                                })?);
                            } else {
                                return Err(Error::UnexpectedAttribute {
                                    name: attr.name.local_name.clone(),
                                    position: reader.position(),
                                });
                            }
                        }

                        if let Some(expr) = expr {
                            let event = reader.next()?;
                            let case = inner_from_xml(&event, reader)?;
                            cases.push((expr, case));
                        } else {
                            return Err(Error::MissingAttribute("expr".to_string()));
                        }
                    }
                    XmlEvent::StartElement {
                        ref name, attributes, ..
                    } if name.local_name == ELEM_DEFAULT => {
                        if default.is_some() {
                            return Err(Error::UnexpectedElement {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }

                        for attr in attributes {
                            return Err(Error::UnexpectedAttribute {
                                name: attr.name.local_name.clone(),
                                position: reader.position(),
                            });
                        }

                        let event = reader.next()?;
                        default = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_MATCH => {
                        if let Some(base) = base {
                            return Ok(Box::new(MatcherConfig::new(base, cases, default)));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_CONCAT => {
            let mut subparts = Vec::new();
            let mut reject = Vec::new();
            let mut joiner = String::new();

            for attr in attributes {
                if attr.name.local_name == "joiner" {
                    joiner = attr.value.clone();
                } else {
                    return Err(Error::UnexpectedAttribute {
                        name: attr.name.local_name.clone(),
                        position: reader.position(),
                    });
                }
            }

            loop {
                let event = reader.next()?;
                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        subparts.push(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::StartElement { name, .. } if name.local_name == "Reject" => loop {
                        match reader.next()? {
                            XmlEvent::Characters(data) => {
                                reject.extend(data.split_whitespace().map(|s| s.to_string()));
                            }
                            XmlEvent::Whitespace(_) => {}
                            XmlEvent::EndElement { name } if name.local_name == "Reject" => {
                                break;
                            }
                            other => {
                                return Err(Error::UnexpectedEvent {
                                    event: other,
                                    position: reader.position(),
                                });
                            }
                        }
                    },
                    XmlEvent::EndElement { name } if name.local_name == ELEM_CONCAT => {
                        return Ok(Box::new(ConcatterConfig::new(subparts, reject).with_joiner(joiner)));
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_LITERAL => {
            let mut literal = String::new();

            for attr in attributes {
                if attr.name.local_name == "text" {
                    literal = attr.value.clone();
                } else {
                    return Err(Error::UnexpectedAttribute {
                        name: attr.name.local_name.clone(),
                        position: reader.position(),
                    });
                }
            }

            loop {
                match reader.next()? {
                    XmlEvent::EndElement { name } if name.local_name == ELEM_LITERAL => {
                        return Ok(Box::new(literal));
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_SWITCH => {
            let mut subparts = Vec::new();

            for attr in attributes {
                return Err(Error::UnexpectedAttribute {
                    name: attr.name.local_name.clone(),
                    position: reader.position(),
                });
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        subparts.push(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_SWITCH => {
                        return Ok(Box::new(SwitcherConfig::new(subparts)));
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_WORDS => {
            let mut options = Vec::new();

            for attr in attributes {
                return Err(Error::UnexpectedAttribute {
                    name: attr.name.local_name.clone(),
                    position: reader.position(),
                });
            }

            loop {
                match reader.next()? {
                    XmlEvent::Characters(data) => {
                        options.extend(data.split_whitespace().map(|s| s.to_string()));
                    }
                    XmlEvent::Whitespace(_) => {}
                    XmlEvent::EndElement { name } if name.local_name == ELEM_WORDS => {
                        options.dedup();
                        return Ok(Box::new(options));
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_OPTION => {
            let mut probability = 0.5;
            let mut subpart = None;

            for attr in attributes {
                if attr.name.local_name == "probability" {
                    probability = attr.value.parse().map_err(|_| Error::InvalidValue {
                        attribute: attr.name.local_name.clone(),
                        value: attr.value.clone(),
                        position: reader.position(),
                    })?;
                } else {
                    return Err(Error::UnexpectedAttribute {
                        name: attr.name.local_name.clone(),
                        position: reader.position(),
                    });
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        if subpart.is_some() {
                            return Err(Error::UnexpectedElement {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }
                        subpart = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_OPTION => {
                        if let Some(subpart) = subpart {
                            return Ok(Box::new(OptionalConfig::new(subpart, probability)));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_REPEAT => {
            let mut min = 1;
            let mut max = 2;
            let mut subpart = None;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    "min" => {
                        min = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    "max" => {
                        max = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    other => {
                        return Err(Error::UnexpectedAttribute {
                            name: other.to_string(),
                            position: reader.position(),
                        });
                    }
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        if subpart.is_some() {
                            return Err(Error::UnexpectedElement {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }
                        subpart = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_REPEAT => {
                        if let Some(subpart) = subpart {
                            if min > max {
                                return Err(Error::InvalidValue {
                                    attribute: "min".to_string(),
                                    value: min.to_string(),
                                    position: reader.position(),
                                });
                            }

                            return Ok(Box::new(RepeaterConfig::new(subpart, min, max)));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_NUMBER => {
            let mut min = 1;
            let mut max = 99;
            let mut style = NumberStyle::Decimal;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    "min" => {
                        min = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    "max" => {
                        max = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    "style" => {
                        style = match attr.value.as_str() {
                            "Dec" | "Decimal" => NumberStyle::Decimal,
                            "Hex" | "HexUpper" | "HexadecimalUpper" => NumberStyle::HexadecimalUpper,
                            "HexLower" | "HexadecimalLower" => NumberStyle::HexadecimalLower,
                            "Oct" | "Octal" => NumberStyle::Octal,
                            "Bin" | "Binary" => NumberStyle::Binary,
                            "Roman" | "RomanUpper" => NumberStyle::RomanUpper,
                            "RomanLower" => NumberStyle::RomanLower,
                            other => {
                                return Err(Error::InvalidValue {
                                    attribute: attr.name.local_name.clone(),
                                    value: other.to_string(),
                                    position: reader.position(),
                                });
                            }
                        };
                    }
                    other => {
                        return Err(Error::UnexpectedAttribute {
                            name: other.to_string(),
                            position: reader.position(),
                        });
                    }
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::EndElement { name } if name.local_name == ELEM_NUMBER => {
                        if min > max {
                            return Err(Error::InvalidValue {
                                attribute: "min".to_string(),
                                value: min.to_string(),
                                position: reader.position(),
                            });
                        }

                        return Ok(Box::new(NumbererConfig::new(min, max).with_style(style)));
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_CAPITALIZE => {
            let mut mode = CapitalizerMode::FirstUpper;
            for attr in attributes {
                match attr.name.local_name.as_str() {
                    "mode" => {
                        mode = match attr.value.as_str() {
                            "AllLower" => CapitalizerMode::AllLower,
                            "FirstUpper" => CapitalizerMode::FirstUpper,
                            "AllUpper" => CapitalizerMode::AllUpper,
                            other => {
                                return Err(Error::InvalidValue {
                                    attribute: attr.name.local_name.clone(),
                                    value: other.to_string(),
                                    position: reader.position(),
                                });
                            }
                        };
                    }
                    other => {
                        return Err(Error::UnexpectedAttribute {
                            name: other.to_string(),
                            position: reader.position(),
                        });
                    }
                }
            }

            let mut subpart = None;

            for attr in attributes {
                return Err(Error::UnexpectedAttribute {
                    name: attr.name.local_name.clone(),
                    position: reader.position(),
                });
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        if subpart.is_some() {
                            return Err(Error::UnexpectedEvent {
                                event,
                                position: reader.position(),
                            });
                        }

                        subpart = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { ref name } if name.local_name == ELEM_CAPITALIZE => {
                        return Ok(Box::new(CapitalizerConfig::new(
                            subpart.ok_or_else(|| Error::UnexpectedEvent {
                                event: event.clone(),
                                position: reader.position(),
                            })?,
                            mode,
                        )));
                    }
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }
        }

        other => {
            return Err(Error::UnexpectedEvent {
                event: other.clone(),
                position: reader.position(),
            });
        }
    }
}

fn parse_reject<R: Read>(reader: &mut EventReader<R>, reject: &mut Vec<String>) -> Result<()> {
    loop {
        match reader.next()? {
            XmlEvent::Characters(data) => {
                reject.extend(data.split_whitespace().map(|s| s.to_string()));
            }
            XmlEvent::Whitespace(_) => {}
            XmlEvent::EndElement { name } if name.local_name == "Reject" => {
                break;
            }
            other => {
                return Err(Error::UnexpectedEvent {
                    event: other,
                    position: reader.position(),
                });
            }
        }
    }

    Ok(())
}

fn parse_tokenizer<R: Read>(
    reader: &mut EventReader<R>,
    elem: &str,
    attributes: &[OwnedAttribute],
) -> Result<Tokenizer> {
    match elem {
        ELEM_SPLIT_TOKENIZER => {
            let mut chars: Vec<char> = Vec::new();
            for attr in attributes {
                if attr.name.local_name == "split_chars" {
                    chars = attr.value.chars().collect();
                } else {
                    return Err(Error::UnexpectedAttribute {
                        name: attr.name.local_name.clone(),
                        position: reader.position(),
                    });
                }
            }
            if chars.is_empty() {
                chars.push('/');
            }
            consume_empty_element(reader, elem)?;
            Ok(Tokenizer::SplitChars(chars))
        }

        ELEM_CHUNK_TOKENIZER => {
            let mut len: Option<usize> = None;
            for attr in attributes {
                if attr.name.local_name == "len" {
                    len = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                        attribute: attr.name.local_name.clone(),
                        value: attr.value.clone(),
                        position: reader.position(),
                    })?);
                } else {
                    return Err(Error::InvalidValue {
                        attribute: attr.name.local_name.clone(),
                        value: attr.value.clone(),
                        position: reader.position(),
                    });
                }
            }

            let len = len.ok_or_else(|| Error::MissingAttribute("len".to_string()))?;

            if len == 0 {
                return Err(Error::InvalidValue {
                    attribute: "len".to_string(),
                    value: "0".to_string(),
                    position: reader.position(),
                });
            }
            consume_empty_element(reader, elem)?;
            Ok(Tokenizer::Chunker(len))
        }

        ELEM_SSP_TOKENIZER => {
            for attr in attributes {
                return Err(Error::UnexpectedAttribute {
                    name: attr.name.local_name.clone(),
                    position: reader.position(),
                });
            }
            let mut ranks = HashMap::new();

            loop {
                match reader.next()? {
                    XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_CLASS => {
                        let mut rank: Option<u8> = None;
                        for attr in &attributes {
                            if attr.name.local_name == "rank" {
                                rank = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                                    attribute: "rank".to_string(),
                                    value: attr.value.clone(),
                                    position: reader.position(),
                                })?);
                            } else {
                                return Err(Error::UnexpectedAttribute {
                                    name: attr.name.local_name.clone(),
                                    position: reader.position(),
                                });
                            }
                        }
                        let rank = rank.ok_or_else(|| Error::MissingAttribute("rank".to_string()))?;

                        loop {
                            match reader.next()? {
                                XmlEvent::Characters(data) => {
                                    for c in data.chars().filter(|c| !c.is_whitespace()) {
                                        ranks.insert(c, rank);
                                    }
                                }
                                XmlEvent::Whitespace(_) => {}
                                XmlEvent::EndElement { name } if name.local_name == ELEM_CLASS => break,
                                other => {
                                    return Err(Error::UnexpectedEvent {
                                        event: other,
                                        position: reader.position(),
                                    });
                                }
                            }
                        }
                    }
                    XmlEvent::Whitespace(_) => {}
                    XmlEvent::EndElement { name } if name.local_name == elem => break,
                    other => {
                        return Err(Error::UnexpectedEvent {
                            event: other,
                            position: reader.position(),
                        });
                    }
                }
            }

            if ranks.is_empty() {
                Ok(Tokenizer::default_ssp())
            } else {
                Ok(Tokenizer::Ssp { ranks })
            }
        }

        other => Err(Error::UnexpectedElement {
            name: other.to_string(),
            position: reader.position(),
        }),
    }
}

fn consume_empty_element<R: Read>(reader: &mut EventReader<R>, elem: &str) -> Result<()> {
    loop {
        match reader.next()? {
            XmlEvent::Whitespace(_) => {}
            XmlEvent::EndElement { name } if name.local_name == elem => return Ok(()),
            other => {
                return Err(Error::UnexpectedEvent {
                    event: other,
                    position: reader.position(),
                });
            }
        }
    }
}

impl Error {
    pub fn position(&self) -> Option<TextPosition> {
        match self {
            Error::Io(_) | Error::Xml(_) | Error::MissingAttribute(_) => None,
            Error::UnexpectedEvent { position, .. }
            | Error::UnexpectedElement { position, .. }
            | Error::UnexpectedEnd { position, .. }
            | Error::UnexpectedAttribute { position, .. }
            | Error::InvalidValue { position, .. }
            | Error::InvalidRegex { position, .. } => Some(*position),
        }
    }
}
