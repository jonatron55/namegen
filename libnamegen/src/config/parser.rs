use std::{
    collections::{HashMap, HashSet},
    io::{Error as IoError, Read},
    mem::take,
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
        GeneratorConfig,
        elements::{
            ATTR_CUTOFF_LEN, ATTR_DISPLAY_NAME, ATTR_EXPR, ATTR_ID, ATTR_LEN, ATTR_MAX, ATTR_MIN, ATTR_MODE,
            ATTR_PROBABILITY, ATTR_RANK, ATTR_REJECT_TRAINING, ATTR_SEP, ATTR_SPLIT_CHARS, ATTR_STYLE, ATTR_TARGET_LEN,
            ATTR_TEXT, ATTR_UNIFORM, ELEM_CAPITALIZE, ELEM_CASE, ELEM_CHUNK_TOKENIZER, ELEM_CLASS, ELEM_DEFAULT,
            ELEM_DESCRIPTION, ELEM_JOIN, ELEM_LITERAL, ELEM_MARKOV, ELEM_MATCH, ELEM_NUMBER, ELEM_OPTION, ELEM_PARAM,
            ELEM_REJECT, ELEM_REPEAT, ELEM_ROOT, ELEM_SPLIT_TOKENIZER, ELEM_SSP_TOKENIZER, ELEM_SWITCH, ELEM_WORDS,
            NS_XSI,
        },
    },
    generator::{CapitalizerMode, NumberStyle, Tokenizer},
};

lazy_static! {
    static ref VALID_PART_TYPES: HashSet<&'static str> = HashSet::from([
        ELEM_CAPITALIZE,
        ELEM_JOIN,
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

    #[error("{event:?} is not expected here")]
    UnexpectedEvent { event: XmlEvent, position: TextPosition },

    #[error("<{name}> is not valid here")]
    UnexpectedElement { name: String, position: TextPosition },

    #[error("</{name}> does not match the opening <{opening_name}>")]
    UnexpectedEnd { name: String, opening_name: String, position: TextPosition },

    #[error("{name} is not a valid attribute for this element")]
    UnexpectedAttribute { name: String, position: TextPosition },

    #[error("\"{value}\" is not a valid value for {attribute}")]
    InvalidValue {
        attribute: String,
        value: String,
        position: TextPosition,
    },

    #[error("The regular expression in attribute {attribute} is invalid: {err}")]
    InvalidRegex {
        attribute: String,
        position: TextPosition,
        err: RegexError,
    },

    #[error("Missing {0} attribute")]
    MissingAttribute(String),

    #[error("Parameter {id} already has a description")]
    DuplicateParameter { id: String, position: TextPosition },

    #[error("Document is too deeply nested")]
    Depth { position: TextPosition },

    #[error("Word length exceeds the maximum allowed")]
    WordLength { position: TextPosition },

    #[error("Word count exceeds the maximum allowed")]
    WordCount { position: TextPosition },
}

const MAX_DEPTH: usize = 64;
const MAX_WORD_LENGTH: usize = 64;
const MAX_WORD_COUNT: usize = 65536;

pub type Result<T> = StdResult<T, Error>;

pub fn from_xml<R: Read>(reader: &mut EventReader<R>) -> Result<GeneratorConfig> {
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
        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_ROOT => {
            for attr in attributes {
                // XSI attributes are allowed on the root element, but nothing else
                if attr.name.namespace.as_deref() != Some(NS_XSI) {
                    return Err(Error::UnexpectedAttribute {
                        name: attr.name.local_name.clone(),
                        position: reader.position(),
                    });
                }
            }
        }
        other => {
            return Err(Error::UnexpectedEvent {
                event: other,
                position: reader.position(),
            });
        }
    }

    let event = reader.next()?;

    let description = if let XmlEvent::StartElement { name, attributes, .. } = &event
        && name.local_name == ELEM_DESCRIPTION
    {
        let mut display_name = "Unnamed Generator".to_string();
        let mut description = String::new();
        let mut arg_display_names = HashMap::new();

        for attr in attributes {
            match attr.name.local_name.as_str() {
                ATTR_DISPLAY_NAME => {
                    display_name = attr.value.clone();
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
                XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_PARAM => {
                    let mut id = None;
                    let mut display_name = String::new();

                    for attr in attributes {
                        match attr.name.local_name.as_str() {
                            ATTR_ID => {
                                id = Some(attr.value.clone());
                            }
                            ATTR_DISPLAY_NAME => {
                                display_name = attr.value.clone();
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
                            XmlEvent::EndElement { name } if name.local_name == ELEM_PARAM => {
                                if let Some(id) = &id {
                                    if arg_display_names.insert(id.clone(), take(&mut display_name)).is_some() {
                                        return Err(Error::DuplicateParameter {
                                            id: id.clone(),
                                            position: reader.position(),
                                        });
                                    }
                                    break;
                                } else {
                                    return Err(Error::MissingAttribute(ATTR_ID.to_string()));
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
                XmlEvent::Characters(data) => {
                    description.push_str(&data);
                }
                XmlEvent::Whitespace(_) => {
                    description.push_str(" ");
                }
                XmlEvent::EndElement { name } if name.local_name == ELEM_DESCRIPTION => {
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

        let event = reader.next()?;
        GeneratorConfig::Description {
            display_name,
            description,
            arg_display_names,
            subpart: inner_from_xml(&event, reader, 0)?,
        }
    } else {
        GeneratorConfig::Description {
            display_name: "Unnamed Generator".to_string(),
            description: String::new(),
            arg_display_names: HashMap::new(),
            subpart: inner_from_xml(&event, reader, 0)?,
        }
    };

    let event = reader.next()?;

    match event {
        XmlEvent::EndElement { name } => {
            if name.local_name == ELEM_ROOT {
                Ok(description)
            } else {
                Err(Error::UnexpectedEnd {
                    name: name.local_name,
                    opening_name: ELEM_ROOT.to_string(),
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

fn inner_from_xml<R: Read>(event: &XmlEvent, reader: &mut EventReader<R>, depth: usize) -> Result<Box<GeneratorConfig>> {
    if depth >= MAX_DEPTH {
        return Err(Error::Depth {
            position: reader.position(),
        });
    }

    match event {
        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_MARKOV => {
            let mut id = None;
            let mut training_data = Vec::new();
            let mut reject = Vec::new();
            let mut reject_training = false;
            let mut uniform = false;
            let mut target_len = None;
            let mut cutoff_len = None;
            let mut tokenizer: Option<Tokenizer> = None;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    ATTR_ID => {
                        id = Some(attr.value.clone());
                    }
                    ATTR_TARGET_LEN => {
                        target_len = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: ATTR_TARGET_LEN.to_string(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?);
                    }
                    ATTR_CUTOFF_LEN => {
                        cutoff_len = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: ATTR_CUTOFF_LEN.to_string(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?);
                    }
                    ATTR_REJECT_TRAINING => {
                        reject_training = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: ATTR_REJECT_TRAINING.to_string(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    ATTR_UNIFORM => {
                        uniform = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: ATTR_UNIFORM.to_string(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    _ => {
                        return Err(Error::UnexpectedAttribute {
                            name: attr.name.local_name.clone(),
                            position: reader.position(),
                        });
                    }
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
                        ELEM_REJECT => parse_reject(reader, &mut reject)?,
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
                        for word in data.split_whitespace() {
                            if word.len() > MAX_WORD_LENGTH {
                                return Err(Error::WordLength {
                                    position: reader.position(),
                                });
                            }

                            if training_data.len() >= MAX_WORD_COUNT {
                                return Err(Error::WordCount {
                                    position: reader.position(),
                                });
                            }

                            training_data.push(word.to_string());
                        }
                    }
                    XmlEvent::EndElement { name } => {
                        if name.local_name == ELEM_MARKOV {
                            let tokenizer = tokenizer.unwrap_or_default();
                            training_data.sort_unstable();
                            training_data.dedup();

                            return Ok(Box::new(GeneratorConfig::Markov {
                                id,
                                data: training_data,
                                target_len,
                                cutoff_len,
                                reject,
                                reject_training,
                                uniform,
                                tokenizer,
                            }));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name,
                                opening_name: ELEM_MARKOV.to_string(),
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
            let mut id = None;
            let mut base = None;
            let mut cases = Vec::new();
            let mut default = None;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    ATTR_ID => {
                        id = Some(attr.value.clone());
                    }
                    _ => {
                        return Err(Error::UnexpectedAttribute {
                            name: attr.name.local_name.clone(),
                            position: reader.position(),
                        });
                    }
                }
            }

            loop {
                let event = reader.next()?;
                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        if base.is_some() {
                            return Err(Error::UnexpectedElement {
                                name: name.local_name.clone(),
                                position: reader.position(),
                            });
                        }

                        base = Some(inner_from_xml(&event, reader, depth + 1)?);
                    }
                    XmlEvent::StartElement {
                        ref name, attributes, ..
                    } if name.local_name == ELEM_CASE => {
                        let mut expr = None;

                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                ATTR_EXPR => {
                                    expr = Some(Regex::new(&attr.value).map_err(|err| Error::InvalidRegex {
                                        attribute: ATTR_EXPR.to_string(),
                                        position: reader.position(),
                                        err,
                                    })?);
                                }
                                _ => {
                                    return Err(Error::UnexpectedAttribute {
                                        name: attr.name.local_name.clone(),
                                        position: reader.position(),
                                    });
                                }
                            }
                        }

                        if let Some(expr) = expr {
                            let event = reader.next()?;
                            let case = inner_from_xml(&event, reader, depth + 1)?;
                            cases.push((expr, case));
                        } else {
                            return Err(Error::MissingAttribute(ATTR_EXPR.to_string()));
                        }

                        let event = reader.next()?;

                        match event {
                            XmlEvent::EndElement { name } if name.local_name == ELEM_CASE => {}
                            other => {
                                return Err(Error::UnexpectedEvent {
                                    event: other,
                                    position: reader.position(),
                                });
                            }
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
                        default = Some(inner_from_xml(&event, reader, depth + 1)?);

                        let event = reader.next()?;

                        match event {
                            XmlEvent::EndElement { name } if name.local_name == ELEM_DEFAULT => {}
                            other => {
                                return Err(Error::UnexpectedEvent {
                                    event: other,
                                    position: reader.position(),
                                });
                            }
                        }
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_MATCH => {
                        if let Some(base) = base {
                            return Ok(Box::new(GeneratorConfig::Matcher {
                                id,
                                base,
                                cases,
                                default,
                            }));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name.clone(),
                                opening_name: ELEM_MATCH.to_string(),
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

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_JOIN => {
            let mut id = None;
            let mut subparts = Vec::new();
            let mut reject = Vec::new();
            let mut sep = String::new();

            for attr in attributes {
                if attr.name.local_name == ATTR_ID {
                    id = Some(attr.value.clone());
                } else if attr.name.local_name == ATTR_SEP {
                    sep = attr.value.clone();
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
                        subparts.push(inner_from_xml(&event, reader, depth + 1)?);
                    }
                    XmlEvent::StartElement { name, .. } if name.local_name == ELEM_REJECT => loop {
                        match reader.next()? {
                            XmlEvent::Characters(data) => {
                                reject.extend(data.split_whitespace().map(|s| s.to_string()));
                            }
                            XmlEvent::Whitespace(_) => {}
                            XmlEvent::EndElement { name } if name.local_name == ELEM_REJECT => {
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
                    XmlEvent::EndElement { name } if name.local_name == ELEM_JOIN => {
                        return Ok(Box::new(GeneratorConfig::Joiner {
                            id,
                            subparts,
                            sep,
                            reject,
                        }));
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
            let mut id = None;
            let mut literal = String::new();

            for attr in attributes {
                if attr.name.local_name == ATTR_ID {
                    id = Some(attr.value.clone());
                } else if attr.name.local_name == ATTR_TEXT {
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
                        return Ok(Box::new(GeneratorConfig::Literal { id, text: literal }));
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
            let mut id = None;
            let mut subparts = Vec::new();

            for attr in attributes {
                if attr.name.local_name == ATTR_ID {
                    id = Some(attr.value.clone());
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
                        subparts.push(inner_from_xml(&event, reader, depth + 1)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_SWITCH => {
                        return Ok(Box::new(GeneratorConfig::Switcher { id, subparts }));
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
            let mut id = None;
            let mut words = Vec::new();

            for attr in attributes {
                if attr.name.local_name == ATTR_ID {
                    id = Some(attr.value.clone());
                } else {
                    return Err(Error::UnexpectedAttribute {
                        name: attr.name.local_name.clone(),
                        position: reader.position(),
                    });
                }
            }

            loop {
                match reader.next()? {
                    XmlEvent::Characters(data) => {
                        words.extend(data.split_whitespace().map(|s| s.to_string()));
                    }
                    XmlEvent::Whitespace(_) => {}
                    XmlEvent::EndElement { name } if name.local_name == ELEM_WORDS => {
                        words.sort_unstable();
                        words.dedup();
                        return Ok(Box::new(GeneratorConfig::Words { id, words }));
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
            let mut id = None;
            let mut probability = 0.5;
            let mut subpart = None;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    ATTR_ID => {
                        id = Some(attr.value.clone());
                    }
                    ATTR_PROBABILITY => {
                        probability = attr.value.parse().map_err(|_| Error::InvalidValue {
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
                        subpart = Some(inner_from_xml(&event, reader, depth + 1)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_OPTION => {
                        if let Some(subpart) = subpart {
                            return Ok(Box::new(GeneratorConfig::Optional {
                                id,
                                generator: subpart,
                                probability,
                            }));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name.clone(),
                                opening_name: ELEM_OPTION.to_string(),
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
            let mut id = None;
            let mut min = 1;
            let mut max = 2;
            let mut subpart = None;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    ATTR_ID => {
                        id = Some(attr.value.clone());
                    }
                    ATTR_MIN => {
                        min = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    ATTR_MAX => {
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
                        subpart = Some(inner_from_xml(&event, reader, depth + 1)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_REPEAT => {
                        if let Some(subpart) = subpart {
                            if min > max {
                                return Err(Error::InvalidValue {
                                    attribute: ATTR_MIN.to_string(),
                                    value: min.to_string(),
                                    position: reader.position(),
                                });
                            }

                            return Ok(Box::new(GeneratorConfig::Repeater {
                                id,
                                generator: subpart,
                                min,
                                max,
                            }));
                        } else {
                            return Err(Error::UnexpectedEnd {
                                name: name.local_name.clone(),
                                opening_name: ELEM_REPEAT.to_string(),
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
            let mut id = None;

            let mut min = 1;
            let mut max = 99;
            let mut style = NumberStyle::Decimal;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    ATTR_ID => {
                        id = Some(attr.value.clone());
                    }
                    ATTR_MIN => {
                        min = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    ATTR_MAX => {
                        max = attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?;
                    }
                    ATTR_STYLE => {
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
                                attribute: ATTR_MIN.to_string(),
                                value: min.to_string(),
                                position: reader.position(),
                            });
                        }

                        return Ok(Box::new(GeneratorConfig::Numberer { id, min, max, style }));
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
            let mut id = None;
            let mut mode = CapitalizerMode::FirstUpper;
            for attr in attributes {
                match attr.name.local_name.as_str() {
                    ATTR_ID => {
                        id = Some(attr.value.clone());
                    }
                    ATTR_MODE => {
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

                        subpart = Some(inner_from_xml(&event, reader, depth + 1)?);
                    }
                    XmlEvent::EndElement { ref name } if name.local_name == ELEM_CAPITALIZE => {
                        return Ok(Box::new(GeneratorConfig::Capitalizer {
                            id,
                            subpart: subpart.ok_or_else(|| Error::UnexpectedEvent {
                                event: event.clone(),
                                position: reader.position(),
                            })?,
                            mode,
                        }));
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
            XmlEvent::EndElement { name } if name.local_name == ELEM_REJECT => {
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
                match attr.name.local_name.as_str() {
                    ATTR_SPLIT_CHARS => {
                        chars = attr.value.chars().collect();
                    }
                    other => {
                        return Err(Error::UnexpectedAttribute {
                            name: other.to_string(),
                            position: reader.position(),
                        });
                    }
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
                match attr.name.local_name.as_str() {
                    ATTR_LEN => {
                        len = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                            attribute: attr.name.local_name.clone(),
                            value: attr.value.clone(),
                            position: reader.position(),
                        })?);
                    }
                    other => {
                        return Err(Error::UnexpectedAttribute {
                            name: other.to_string(),
                            position: reader.position(),
                        });
                    }
                }
            }

            let len = len.ok_or_else(|| Error::MissingAttribute(ATTR_LEN.to_string()))?;

            if len == 0 {
                return Err(Error::InvalidValue {
                    attribute: ATTR_LEN.to_string(),
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
                            if attr.name.local_name == ATTR_RANK {
                                rank = Some(attr.value.parse().map_err(|_| Error::InvalidValue {
                                    attribute: ATTR_RANK.to_string(),
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
                        let rank = rank.ok_or_else(|| Error::MissingAttribute(ATTR_RANK.to_string()))?;

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
            | Error::InvalidRegex { position, .. }
            | Error::DuplicateParameter { position, .. }
            | Error::Depth { position }
            | Error::WordLength { position }
            | Error::WordCount { position } => Some(*position),
        }
    }
}
