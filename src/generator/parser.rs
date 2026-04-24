use std::{error::Error, io::Read};

use xml::{
    attribute::OwnedAttribute,
    reader::{EventReader, XmlEvent},
};

use crate::generator::{
    Generator,
    concatter::Concatter,
    markov::{MarkovGen, Tokenizer},
    numberer::{NumberStyle, Numberer},
    optional::Optional,
    repeater::Repeater,
    switcher::Switcher,
};

const ELEM_MARKOV: &str = "Markov";
const ELEM_CONCAT: &str = "Concat";
const ELEM_SWITCH: &str = "Switch";
const ELEM_WORDS: &str = "Words";
const ELEM_OPTION: &str = "Option";
const ELEM_REPEAT: &str = "Repeat";
const ELEM_NUMBER: &str = "Number";

const ELEM_SPLIT_TOKENIZER: &str = "SplitTokenizer";
const ELEM_SPLIT_LEN_TOKENIZER: &str = "SplitLenTokenizer";
const ELEM_SSP_TOKENIZER: &str = "SspTokenizer";
const ELEM_CLASS: &str = "Class";

const VALID_PART_TYPES: [&str; 7] = [
    ELEM_MARKOV,
    ELEM_CONCAT,
    ELEM_SWITCH,
    ELEM_WORDS,
    ELEM_OPTION,
    ELEM_REPEAT,
    ELEM_NUMBER,
];

pub fn from_xml<R: Read>(reader: &mut EventReader<R>) -> Result<Box<dyn Generator>, Box<dyn Error>> {
    let event = reader.next()?;

    match event {
        XmlEvent::StartDocument { .. } => {}
        _ => {
            return Err("Expected start document".into());
        }
    }

    let event = reader.next()?;

    match event {
        XmlEvent::StartElement { name, .. } if name.local_name == "NameGen" => {}
        other => {
            return Err(format!("Expected <NameGen> element, encountered {other:?}").into());
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
                Err(format!("Unexpected end element: </{}>", name).into())
            }
        }
        other => Err(format!("Unexpected event: {other:?}").into()),
    }
}

fn inner_from_xml<R: Read>(
    event: &XmlEvent,
    reader: &mut EventReader<R>,
) -> Result<Box<dyn Generator>, Box<dyn Error>> {
    match event {
        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_MARKOV => {
            let mut training_data = Vec::new();
            let mut reject = Vec::new();
            let mut reject_training = false;
            let mut case_insensitive = false;
            let mut target_len = None;
            let mut tokenizer: Option<Tokenizer> = None;

            for attr in attributes {
                if attr.name.local_name == "target_len" {
                    target_len = Some(
                        attr.value
                            .parse()
                            .map_err(|_| format!("Invalid target_len value: {}", attr.value))?,
                    );
                } else if attr.name.local_name == "reject_training" {
                    reject_training = attr
                        .value
                        .parse()
                        .map_err(|_| format!("Invalid reject_training value: {}", attr.value))?;
                } else if attr.name.local_name == "case_insensitive" {
                    case_insensitive = attr
                        .value
                        .parse()
                        .map_err(|_| format!("Invalid case_insensitive value: {}", attr.value))?;
                } else {
                    return Err(format!("Unexpected attribute: {}", attr.name).into());
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
                        ELEM_SPLIT_TOKENIZER | ELEM_SPLIT_LEN_TOKENIZER | ELEM_SSP_TOKENIZER => {
                            if tokenizer.is_some() {
                                return Err("Multiple tokenizer elements in <Markov>".into());
                            }
                            tokenizer = Some(parse_tokenizer(reader, &name.local_name, attributes)?);
                        }
                        _ => {
                            return Err(format!("Unexpected element: <{name}>").into());
                        }
                    },
                    XmlEvent::Characters(data) => {
                        training_data.extend(data.split_whitespace().map(|s| s.to_string()));
                    }
                    XmlEvent::EndElement { name } => {
                        if name.local_name == ELEM_MARKOV {
                            let tokenizer = tokenizer.unwrap_or_default();

                            if reject_training {
                                reject.extend_from_slice(&training_data);
                                reject.dedup();
                            }

                            return Ok(Box::new(MarkovGen::train(
                                &training_data,
                                target_len,
                                reject,
                                &tokenizer,
                                case_insensitive,
                            )));
                        } else {
                            return Err(format!("Unexpected end element: </{}>", name).into());
                        }
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
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
                    return Err(format!("Unexpected attribute: {}", attr.name).into());
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
                                return Err(format!("Unexpected event: {other:?}").into());
                            }
                        }
                    },
                    XmlEvent::EndElement { name } if name.local_name == ELEM_CONCAT => {
                        return Ok(Box::new(Concatter::new(subparts, reject).with_joiner(joiner)));
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_SWITCH => {
            let mut subparts = Vec::new();

            for attr in attributes {
                return Err(format!("Unexpected attribute: {}", attr.name).into());
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        subparts.push(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_SWITCH => {
                        return Ok(Box::new(Switcher::new(subparts)));
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_WORDS => {
            let mut options = Vec::new();

            for attr in attributes {
                return Err(format!("Unexpected attribute: {}", attr.name).into());
            }

            loop {
                match reader.next()? {
                    XmlEvent::Characters(data) => {
                        options.extend(data.split_whitespace().map(|s| s.to_string()));
                    }
                    XmlEvent::Whitespace(_) => {}
                    XmlEvent::EndElement { name } if name.local_name == ELEM_WORDS => {
                        return Ok(Box::new(options));
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
                    }
                }
            }
        }

        XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_OPTION => {
            let mut probability = 0.5;
            let mut subpart = None;

            for attr in attributes {
                if attr.name.local_name == "probability" {
                    probability = attr
                        .value
                        .parse()
                        .map_err(|_| format!("Invalid probability value: {}", attr.value))?;
                } else {
                    return Err(format!("Unexpected attribute: {}", attr.name).into());
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        if subpart.is_some() {
                            return Err("Option elements must contain exactly one generator".into());
                        }
                        subpart = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_OPTION => {
                        if let Some(subpart) = subpart {
                            return Ok(Box::new(Optional::new(subpart, probability)));
                        } else {
                            return Err("Option elements must contain exactly one generator".into());
                        }
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
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
                        min = attr.value.parse().map_err(|_| format!("Invalid min value: {}", attr.value))?;
                    }
                    "max" => {
                        max = attr.value.parse().map_err(|_| format!("Invalid max value: {}", attr.value))?;
                    }
                    other => {
                        return Err(format!("Unexpected attribute: {other}").into());
                    }
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } if VALID_PART_TYPES.contains(&name.local_name.as_str()) => {
                        if subpart.is_some() {
                            return Err("Repeat elements must contain exactly one generator".into());
                        }
                        subpart = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == ELEM_REPEAT => {
                        if let Some(subpart) = subpart {
                            return Ok(Box::new(Repeater::new(subpart, min, max)));
                        } else {
                            return Err("Repeat elements must contain exactly one generator".into());
                        }
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
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
                        min = attr.value.parse().map_err(|_| format!("Invalid min value: {}", attr.value))?;
                    }
                    "max" => {
                        max = attr.value.parse().map_err(|_| format!("Invalid max value: {}", attr.value))?;
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
                            other => return Err(format!("Invalid style value: {other}").into()),
                        };
                    }
                    other => {
                        return Err(format!("Unexpected attribute: {other}").into());
                    }
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::EndElement { name } if name.local_name == ELEM_NUMBER => {
                        return Ok(Box::new(Numberer::new(min, max).with_style(style)));
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
                    }
                }
            }
        }

        other => {
            return Err(format!("Unexpected event: {other:?}").into());
        }
    }
}

fn parse_reject<R: Read>(reader: &mut EventReader<R>, reject: &mut Vec<String>) -> Result<(), Box<dyn Error>> {
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
                return Err(format!("Unexpected event: {other:?}").into());
            }
        }
    }

    Ok(())
}

fn parse_tokenizer<R: Read>(
    reader: &mut EventReader<R>,
    elem: &str,
    attributes: &[OwnedAttribute],
) -> Result<Tokenizer, Box<dyn Error>> {
    match elem {
        ELEM_SPLIT_TOKENIZER => {
            let mut chars: Vec<char> = Vec::new();
            for attr in attributes {
                if attr.name.local_name == "split_chars" {
                    chars = attr.value.chars().collect();
                } else {
                    return Err(format!("Unexpected attribute on <{elem}>: {}", attr.name).into());
                }
            }
            if chars.is_empty() {
                chars.push('/');
            }
            consume_empty_element(reader, elem)?;
            Ok(Tokenizer::SplitChars(chars))
        }

        ELEM_SPLIT_LEN_TOKENIZER => {
            let mut len: Option<usize> = None;
            for attr in attributes {
                if attr.name.local_name == "len" {
                    len = Some(attr.value.parse().map_err(|_| format!("Invalid len value: {}", attr.value))?);
                } else {
                    return Err(format!("Unexpected attribute on <{elem}>: {}", attr.name).into());
                }
            }
            let len = len.ok_or_else(|| format!("<{elem}> requires a len attribute"))?;
            if len == 0 {
                return Err(format!("<{elem}> len must be greater than zero").into());
            }
            consume_empty_element(reader, elem)?;
            Ok(Tokenizer::SplitLen(len))
        }

        ELEM_SSP_TOKENIZER => {
            for attr in attributes {
                return Err(format!("Unexpected attribute on <{elem}>: {}", attr.name).into());
            }
            let mut tokenizer = Tokenizer::default_ssp();
            let Tokenizer::Ssp { ref mut ranks } = tokenizer else {
                unreachable!("default_ssp must return Tokenizer::Ssp");
            };

            loop {
                match reader.next()? {
                    XmlEvent::StartElement { name, attributes, .. } if name.local_name == ELEM_CLASS => {
                        let mut rank: Option<u8> = None;
                        for attr in &attributes {
                            if attr.name.local_name == "rank" {
                                rank = Some(
                                    attr.value.parse().map_err(|_| format!("Invalid rank value: {}", attr.value))?,
                                );
                            } else {
                                return Err(format!("Unexpected attribute on <{ELEM_CLASS}>: {}", attr.name).into());
                            }
                        }
                        let rank = rank.ok_or_else(|| format!("<{ELEM_CLASS}> requires a rank attribute"))?;

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
                                    return Err(format!("Unexpected event inside <{ELEM_CLASS}>: {other:?}").into());
                                }
                            }
                        }
                    }
                    XmlEvent::Whitespace(_) => {}
                    XmlEvent::EndElement { name } if name.local_name == elem => break,
                    other => {
                        return Err(format!("Unexpected event inside <{elem}>: {other:?}").into());
                    }
                }
            }

            Ok(tokenizer)
        }

        _ => Err(format!("Unknown tokenizer element: <{elem}>").into()),
    }
}

fn consume_empty_element<R: Read>(reader: &mut EventReader<R>, elem: &str) -> Result<(), Box<dyn Error>> {
    loop {
        match reader.next()? {
            XmlEvent::Whitespace(_) => {}
            XmlEvent::EndElement { name } if name.local_name == elem => return Ok(()),
            other => {
                return Err(format!("<{elem}> must be empty; unexpected event: {other:?}").into());
            }
        }
    }
}
