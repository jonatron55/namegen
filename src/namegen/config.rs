use std::{error::Error, io::Read};

use serde_derive::{Deserialize, Serialize};
use xml::reader::{EventReader, XmlEvent};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub parts: Vec<PartConfig>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PartConfig {
    MarkovPart {
        training_data: Vec<String>,
        reject: Vec<String>,
    },
    ConcatPart {
        subparts: Vec<String>,
        reject: Vec<String>,
    },
}

impl Config {
    pub fn from_xml<R: Read>(reader: &mut EventReader<R>) -> Result<Self, Box<dyn Error>> {
        let mut parts = Vec::new();
        let mut gen_name: Option<String> = None;
        let event = reader.next()?;
        match event {
            XmlEvent::StartDocument { .. } => {}
            _ => {
                return Err("Expected start document".into());
            }
        }
        let event = reader.next()?;

        match event {
            XmlEvent::StartElement {
                name, attributes, ..
            } if name.local_name == "NameGen" => {
                for attr in attributes {
                    if attr.name.local_name == "name" {
                        gen_name = Some(attr.value);
                    } else {
                        return Err(format!("Unexpected attribute: {}", attr.name).into());
                    }
                }
            }
            other => {
                return Err(format!("Expected <NameGen> element, encountered {other:?}").into());
            }
        }

        loop {
            let event = reader.next()?;

            match event {
                XmlEvent::StartElement { ref name, .. } => {
                    if name.local_name == "Markov" || name.local_name == "Concat" {
                        parts.push(PartConfig::from_xml(&event, reader)?);
                    } else {
                        return Err(format!("Unexpected element: <{}>", name).into());
                    }
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == "NameGen" {
                        break;
                    } else {
                        return Err(format!("Unexpected end element: </{}>", name).into());
                    }
                }
                other => {
                    return Err(format!("Unexpected event: {other:?}").into());
                }
            }
        }

        Ok(Self {
            parts,
            name: gen_name.unwrap_or("Untitled".to_string()),
        })
    }
}

impl PartConfig {
    pub fn from_xml<R: Read>(
        event: &XmlEvent,
        reader: &mut EventReader<R>,
    ) -> Result<Self, Box<dyn Error>> {
        match event {
            XmlEvent::StartElement { name, .. } if name.local_name == "Markov" => {
                let mut training_data = Vec::new();
                let mut reject = Vec::new();

                loop {
                    let event = reader.next()?;

                    match event {
                        XmlEvent::StartElement { ref name, .. } => match name.local_name.as_str() {
                            "Reject" => {
                                let event = reader.next()?;
                                match event {
                                    XmlEvent::Characters(data) => {
                                        reject
                                            .extend(data.split_whitespace().map(|s| s.to_string()));
                                    }
                                    _ => {
                                        return Err("Expected text data".into());
                                    }
                                }

                                let event = reader.next()?;
                                match event {
                                    XmlEvent::EndElement { name }
                                        if name.local_name == "Reject" => {}
                                    _ => {
                                        return Err("Expected </Reject>".into());
                                    }
                                }
                            }
                            _ => {
                                return Err(format!("Unexpected element: <{name}>").into());
                            }
                        },
                        XmlEvent::Characters(data) => {
                            training_data.extend(data.split_whitespace().map(|s| s.to_string()));
                        }
                        XmlEvent::EndElement { name } => {
                            if name.local_name == "Markov" {
                                return Ok(Self::MarkovPart {
                                    training_data,
                                    reject,
                                });
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

            XmlEvent::StartElement { name, .. } if name.local_name == "Concat" => loop {
                let mut subparts = Vec::new();
                let mut reject = Vec::new();
                loop{
                    match reader.next()? {
                        XmlEvent::StartElement { name, .. } if name.local_name == "Part" => {
                            let mut subpart = String::new();
                            loop {
                                match reader.next()? {
                                    XmlEvent::Characters(data) => {
                                        subpart.push_str(&data);
                                    }
                                    XmlEvent::Whitespace(_) => {
                                        subpart.push(' ');
                                    }
                                    XmlEvent::EndElement { name } if name.local_name == "Part" => {
                                        break;
                                    }
                                    other => {
                                        return Err(format!("Unexpected event: {other:?}").into());
                                    }
                                }
                            }

                            subparts.push(subpart);
                        }
                        XmlEvent::StartElement { name, .. } if name.local_name == "Reject" => {
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
                        }
                        XmlEvent::EndElement { name } if name.local_name == "Concat" => {
                            return Ok(Self::ConcatPart { subparts, reject });
                        }
                        other => {
                            return Err(format!("Unexpected event: {other:?}").into());
                        }
                    }
                }
            },

            other => {
                return Err(format!("Unexpected event: {other:?}").into());
            }
        }
    }
}
