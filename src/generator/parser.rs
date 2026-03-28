use std::{error::Error, io::Read};

use xml::reader::{EventReader, XmlEvent};

use crate::generator::{
    Generator, concatter::Concatter, markov::MarkovGen, optional::Optional, repeater::Repeater,
    switcher::Switcher,
};

const VALID_PART_TYPES: [&str; 6] = ["Markov", "Concat", "Switch", "Words", "Option", "Repeat"];

pub fn from_xml<R: Read>(
    reader: &mut EventReader<R>,
) -> Result<Box<dyn Generator>, Box<dyn Error>> {
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
        XmlEvent::StartElement {
            name, attributes, ..
        } if name.local_name == "Markov" => {
            let mut training_data = Vec::new();
            let mut reject = Vec::new();
            let mut target_len = None;

            for attr in attributes {
                if attr.name.local_name == "target_len" {
                    target_len = Some(
                        attr.value
                            .parse()
                            .map_err(|_| format!("Invalid target_len value: {}", attr.value))?,
                    );
                } else {
                    return Err(format!("Unexpected attribute: {}", attr.name).into());
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. } => match name.local_name.as_str() {
                        "Reject" => parse_reject(reader, &mut reject)?,
                        _ => {
                            return Err(format!("Unexpected element: <{name}>").into());
                        }
                    },
                    XmlEvent::Characters(data) => {
                        training_data.extend(data.split_whitespace().map(|s| s.to_string()));
                    }
                    XmlEvent::EndElement { name } => {
                        if name.local_name == "Markov" {
                            return Ok(Box::new(MarkovGen::train(
                                &training_data,
                                target_len,
                                reject,
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

        XmlEvent::StartElement {
            name, attributes, ..
        } if name.local_name == "Concat" => loop {
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
                    XmlEvent::StartElement { ref name, .. }
                        if VALID_PART_TYPES.contains(&name.local_name.as_str()) =>
                    {
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
                    XmlEvent::EndElement { name } if name.local_name == "Concat" => {
                        return Ok(Box::new(
                            Concatter::new(subparts, reject).with_joiner(joiner),
                        ));
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
                    }
                }
            }
        },

        XmlEvent::StartElement {
            name, attributes, ..
        } if name.local_name == "Switch" => {
            let mut subparts = Vec::new();

            for attr in attributes {
                return Err(format!("Unexpected attribute: {}", attr.name).into());
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. }
                        if VALID_PART_TYPES.contains(&name.local_name.as_str()) =>
                    {
                        subparts.push(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == "Switch" => {
                        return Ok(Box::new(Switcher::new(subparts)));
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
                    }
                }
            }
        }

        XmlEvent::StartElement { name, .. } if name.local_name == "Words" => {
            let mut options = Vec::new();

            loop {
                match reader.next()? {
                    XmlEvent::Characters(data) => {
                        options.extend(data.split_whitespace().map(|s| s.to_string()));
                    }
                    XmlEvent::Whitespace(_) => {}
                    XmlEvent::EndElement { name } if name.local_name == "Words" => {
                        return Ok(Box::new(options));
                    }
                    other => {
                        return Err(format!("Unexpected event: {other:?}").into());
                    }
                }
            }
        }

        XmlEvent::StartElement {
            name, attributes, ..
        } if name.local_name == "Option" => {
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
                    XmlEvent::StartElement { ref name, .. }
                        if VALID_PART_TYPES.contains(&name.local_name.as_str()) =>
                    {
                        if subpart.is_some() {
                            return Err("Option elements must contain exactly one generator".into());
                        }
                        subpart = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == "Option" => {
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

        XmlEvent::StartElement {
            name, attributes, ..
        } if name.local_name == "Repeat" => {
            let mut min = 1;
            let mut max = 2;
            let mut subpart = None;

            for attr in attributes {
                match attr.name.local_name.as_str() {
                    "min" => {
                        min = attr
                            .value
                            .parse()
                            .map_err(|_| format!("Invalid min value: {}", attr.value))?;
                    }
                    "max" => {
                        max = attr
                            .value
                            .parse()
                            .map_err(|_| format!("Invalid max value: {}", attr.value))?;
                    }
                    other => {
                        return Err(format!("Unexpected attribute: {other}").into());
                    }
                }
            }

            loop {
                let event = reader.next()?;

                match event {
                    XmlEvent::StartElement { ref name, .. }
                        if VALID_PART_TYPES.contains(&name.local_name.as_str()) =>
                    {
                        if subpart.is_some() {
                            return Err("Repeat elements must contain exactly one generator".into());
                        }
                        subpart = Some(inner_from_xml(&event, reader)?);
                    }
                    XmlEvent::EndElement { name } if name.local_name == "Repeat" => {
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

        other => {
            return Err(format!("Unexpected event: {other:?}").into());
        }
    }
}

fn parse_reject<R: Read>(
    reader: &mut EventReader<R>,
    reject: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
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
