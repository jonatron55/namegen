use std::{collections::HashMap, io::Write};

use crate::{
    config::{
        GeneratorConfig, WriteError,
        elements::{
            ATTR_CUTOFF_LEN, ATTR_DISPLAY_NAME, ATTR_EXPR, ATTR_ID, ATTR_LEN, ATTR_MAX, ATTR_MIN, ATTR_MODE,
            ATTR_PROBABILITY, ATTR_RANK, ATTR_REJECT_TRAINING, ATTR_SCHEMA_LOCATION, ATTR_SEP, ATTR_SPLIT_CHARS,
            ATTR_STYLE, ATTR_TARGET_LEN, ATTR_TEXT, ATTR_UNIFORM, ELEM_CAPITALIZE, ELEM_CASE, ELEM_CHUNK_TOKENIZER,
            ELEM_CLASS, ELEM_DEFAULT, ELEM_DESCRIPTION, ELEM_JOIN, ELEM_LITERAL, ELEM_MARKOV, ELEM_MATCH, ELEM_NUMBER,
            ELEM_OPTION, ELEM_PARAM, ELEM_REJECT, ELEM_REPEAT, ELEM_ROOT, ELEM_SPLIT_TOKENIZER, ELEM_SSP_TOKENIZER,
            ELEM_SWITCH, ELEM_WORDS, NS_XSI, PREFIX_XSI, SCHEMA_LOCATION,
        },
    },
    generator::{CapitalizerMode, Tokenizer},
};
use itertools::Itertools;
use xml::{EventWriter as XmlWriter, name::Name, writer::XmlEvent};

const WRAP_WIDTH: usize = 80;

pub trait WriteXml: Sized {
    fn write_xml(self, writer: &mut XmlWriter<&mut Box<dyn Write>>, indent: usize) -> Result<(), WriteError>;

    fn write_xml_root(self, writer: &mut XmlWriter<&mut Box<dyn Write>>) -> Result<(), WriteError> {
        writer.write(XmlEvent::start_element(ELEM_ROOT).ns(PREFIX_XSI, NS_XSI).attr(
            Name::qualified(ATTR_SCHEMA_LOCATION, NS_XSI, Some(PREFIX_XSI)),
            SCHEMA_LOCATION,
        ))?;

        self.write_xml(writer, 2)?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

impl WriteXml for GeneratorConfig {
    fn write_xml(self, writer: &mut XmlWriter<&mut Box<dyn Write>>, indent: usize) -> Result<(), WriteError> {
        match self {
            GeneratorConfig::Description {
                display_name,
                description,
                arg_display_names,
                subpart,
            } => {
                writer.write(XmlEvent::start_element(ELEM_DESCRIPTION).attr(ATTR_DISPLAY_NAME, &display_name))?;

                for id in arg_display_names.keys().sorted_unstable() {
                    let display_name = &arg_display_names[id];
                    writer.write(
                        XmlEvent::start_element(ELEM_PARAM)
                            .attr(ATTR_ID, id)
                            .attr(ATTR_DISPLAY_NAME, display_name),
                    )?;
                    writer.write(XmlEvent::end_element())?;
                }

                let desc_words = description.split_whitespace();
                write_indented_lines(desc_words, indent + 2, writer)?;

                writer.write(XmlEvent::end_element())?;
                subpart.write_xml(writer, indent)?;
            }

            GeneratorConfig::Capitalizer { id, subpart, mode } => {
                if matches!(mode, CapitalizerMode::FirstUpper) {
                    writer.write(XmlEvent::start_element(ELEM_CAPITALIZE))?;
                } else {
                    let mut ev = XmlEvent::start_element(ELEM_CAPITALIZE);
                    if let Some(id) = &id {
                        ev = ev.attr(ATTR_ID, id);
                    }
                    writer.write(ev.attr(ATTR_MODE, &format!("{:?}", mode)))?;
                }
                subpart.write_xml(writer, indent + 2)?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Joiner {
                id,
                subparts,
                sep,
                mut reject,
            } => {
                let mut ev = XmlEvent::start_element(ELEM_JOIN);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }

                if sep.len() > 0 {
                    writer.write(ev.attr(ATTR_SEP, &sep))?;
                } else {
                    writer.write(ev)?;
                }

                for subpart in subparts {
                    subpart.write_xml(writer, indent + 2)?;
                }

                if reject.len() > 0 {
                    reject.sort_unstable();
                    reject.dedup();
                    writer.write(XmlEvent::start_element(ELEM_REJECT))?;
                    write_indented_lines(&reject, indent + 4, writer)?;
                    writer.write(XmlEvent::end_element())?;
                }

                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Literal { id, text } => {
                let mut ev = XmlEvent::start_element(ELEM_LITERAL);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }

                writer.write(ev.attr(ATTR_TEXT, &text))?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Markov {
                id,
                data,
                target_len,
                cutoff_len,
                reject,
                uniform,
                reject_training,
                tokenizer,
            } => {
                let mut ev = XmlEvent::start_element(ELEM_MARKOV);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }

                let target_len_str: String;
                let cutoff_len_str: String;

                if let Some(target_len) = target_len {
                    target_len_str = target_len.to_string();
                    ev = ev.attr(ATTR_TARGET_LEN, &target_len_str);
                }

                if let Some(cutoff_len) = cutoff_len {
                    cutoff_len_str = cutoff_len.to_string();
                    ev = ev.attr(ATTR_CUTOFF_LEN, &cutoff_len_str);
                }

                if uniform {
                    ev = ev.attr(ATTR_UNIFORM, "true");
                }

                if reject_training {
                    ev = ev.attr(ATTR_REJECT_TRAINING, "true");
                }

                writer.write(ev)?;

                if tokenizer != Tokenizer::default_ssp() {
                    match tokenizer {
                        Tokenizer::SplitChars(chars) => {
                            let chars_str = chars.into_iter().collect::<String>();
                            writer.write(
                                XmlEvent::start_element(ELEM_SPLIT_TOKENIZER).attr(ATTR_SPLIT_CHARS, &chars_str),
                            )?;
                            writer.write(XmlEvent::end_element())?;
                        }
                        Tokenizer::Chunker(len) => {
                            let len_str = len.to_string();
                            writer.write(XmlEvent::start_element(ELEM_CHUNK_TOKENIZER).attr(ATTR_LEN, &len_str))?;
                            writer.write(XmlEvent::end_element())?;
                        }
                        Tokenizer::Ssp { ranks } => {
                            writer.write(XmlEvent::start_element(ELEM_SSP_TOKENIZER))?;

                            let mut classes = HashMap::new();

                            for (ch, rank) in ranks.into_iter() {
                                classes.entry(rank as usize - 1).or_insert_with(String::new).push(ch);
                            }

                            for class in classes.values_mut() {
                                *class = class.chars().sorted_unstable().dedup().collect();
                            }

                            for rank in classes.keys().sorted_unstable().rev() {
                                let class = &classes[rank];
                                if class.len() > 0 {
                                    writer.write(
                                        XmlEvent::start_element(ELEM_CLASS).attr(ATTR_RANK, &(rank + 1).to_string()),
                                    )?;
                                    writer.write(XmlEvent::characters(&class))?;
                                    writer.write(XmlEvent::end_element())?;
                                }
                            }

                            writer.write(XmlEvent::end_element())?;
                        }
                    }
                }

                if reject.len() > 0 {
                    writer.write(XmlEvent::start_element(ELEM_REJECT))?;
                    write_indented_lines(&reject, indent + 4, writer)?;
                    writer.write(XmlEvent::end_element())?;
                }

                write_indented_lines(&data, indent + 2, writer)?;

                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Matcher {
                id,
                base,
                cases,
                default,
            } => {
                let mut ev = XmlEvent::start_element(ELEM_MATCH);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }
                writer.write(ev)?;
                base.write_xml(writer, indent + 2)?;

                for (regex, config) in cases {
                    writer.write(XmlEvent::start_element(ELEM_CASE).attr(ATTR_EXPR, &regex.as_str()))?;
                    config.write_xml(writer, indent + 4)?;
                    writer.write(XmlEvent::end_element())?;
                }

                if let Some(default) = default {
                    writer.write(XmlEvent::start_element(ELEM_DEFAULT))?;
                    default.write_xml(writer, indent + 2)?;
                    writer.write(XmlEvent::end_element())?;
                }

                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Numberer { id, style, min, max } => {
                let mut ev = XmlEvent::start_element(ELEM_NUMBER);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }
                writer.write(
                    ev.attr(ATTR_STYLE, &format!("{:?}", style))
                        .attr(ATTR_MIN, &min.to_string())
                        .attr(ATTR_MAX, &max.to_string()),
                )?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Optional {
                id,
                generator,
                probability,
            } => {
                let mut ev = XmlEvent::start_element(ELEM_OPTION);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }

                writer.write(ev.attr(ATTR_PROBABILITY, &format!("{}", probability)))?;
                generator.write_xml(writer, indent + 2)?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Repeater {
                id,
                generator,
                min,
                max,
            } => {
                let mut ev = XmlEvent::start_element(ELEM_REPEAT);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }

                writer.write(ev.attr(ATTR_MIN, &min.to_string()).attr(ATTR_MAX, &max.to_string()))?;
                generator.write_xml(writer, indent + 2)?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Switcher { id, subparts } => {
                let mut ev = XmlEvent::start_element(ELEM_SWITCH);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }
                writer.write(ev)?;

                for subpart in subparts {
                    subpart.write_xml(writer, indent + 2)?;
                }

                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Words { id, words } => {
                let mut ev = XmlEvent::start_element(ELEM_WORDS);
                if let Some(id) = &id {
                    ev = ev.attr(ATTR_ID, id);
                }

                writer.write(ev)?;
                write_indented_lines(&words, indent + 2, writer)?;
                writer.write(XmlEvent::end_element())?;
            }
        }
        Ok(())
    }
}

fn write_indented_lines(
    words: impl IntoIterator<Item = impl AsRef<str>>,
    indent: usize,
    writer: &mut XmlWriter<&mut Box<dyn Write>>,
) -> Result<(), WriteError> {
    let indent_str = " ".repeat(indent);
    writer.write(XmlEvent::characters("\n"))?;
    writer.write(XmlEvent::characters(&indent_str))?;

    let mut line = String::with_capacity(WRAP_WIDTH);

    for word in words {
        let word = word.as_ref();
        if line.len() > 0 && line.len() + word.len() + 1 > WRAP_WIDTH {
            writer.write(XmlEvent::characters(&line))?;
            writer.write(XmlEvent::characters("\n"))?;
            writer.write(XmlEvent::characters(&indent_str))?;

            line.clear();
        }

        if !line.is_empty() {
            line.push(' ');
        }

        line.push_str(word);
    }

    if !line.is_empty() {
        writer.write(XmlEvent::characters(&line))?;
        writer.write(XmlEvent::characters("\n"))?;
        let indent_str = " ".repeat(indent.saturating_sub(2));
        writer.write(XmlEvent::characters(&indent_str))?;
    }

    Ok(())
}
