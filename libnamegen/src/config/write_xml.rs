use std::{collections::HashMap, io::Write};

use crate::{
    config::{GeneratorConfig, WriteError},
    generator::{CapitalizerMode, Tokenizer},
};
use itertools::Itertools;
use xml::{EventWriter as XmlWriter, writer::XmlEvent};

const WRAP_WIDTH: usize = 80;

pub trait WriteXml {
    fn write_xml(self, writer: &mut XmlWriter<&mut Box<dyn Write>>, indent: usize) -> Result<(), WriteError>;
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
                writer.write(XmlEvent::start_element("Description").attr("display_name", &display_name))?;

                for id in arg_display_names.keys().sorted_unstable() {
                    let display_name = &arg_display_names[id];
                    writer.write(
                        XmlEvent::start_element("Param")
                            .attr("id", id)
                            .attr("display_name", display_name),
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
                    writer.write(XmlEvent::start_element("Capitalize"))?;
                } else {
                    let mut ev = XmlEvent::start_element("Capitalize");
                    if let Some(id) = &id {
                        ev = ev.attr("id", id);
                    }
                    writer.write(ev.attr("mode", &format!("{:?}", mode)))?;
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
                let mut ev = XmlEvent::start_element("Join");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }

                if sep.len() > 0 {
                    writer.write(ev.attr("sep", &sep))?;
                } else {
                    writer.write(ev)?;
                }

                for subpart in subparts {
                    subpart.write_xml(writer, indent + 2)?;
                }

                if reject.len() > 0 {
                    reject.sort_unstable();
                    reject.dedup();
                    writer.write(XmlEvent::start_element("Reject"))?;
                    write_indented_lines(&reject, indent + 4, writer)?;
                    writer.write(XmlEvent::end_element())?;
                }

                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Literal { id, text } => {
                let mut ev = XmlEvent::start_element("Literal");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }

                writer.write(ev.attr("text", &text))?;
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
                let mut ev = XmlEvent::start_element("Markov");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }

                let target_len_str: String;
                let cutoff_len_str: String;

                if let Some(target_len) = target_len {
                    target_len_str = target_len.to_string();
                    ev = ev.attr("target_len", &target_len_str);
                }

                if let Some(cutoff_len) = cutoff_len {
                    cutoff_len_str = cutoff_len.to_string();
                    ev = ev.attr("cutoff_len", &cutoff_len_str);
                }

                if uniform {
                    ev = ev.attr("uniform", "true");
                }

                if reject_training {
                    ev = ev.attr("reject_training", "true");
                }

                writer.write(ev)?;

                if tokenizer != Tokenizer::default_ssp() {
                    match tokenizer {
                        Tokenizer::SplitChars(chars) => {
                            let chars_str = chars.into_iter().collect::<String>();
                            writer.write(XmlEvent::start_element("SplitTokenizer").attr("split_chars", &chars_str))?;
                            writer.write(XmlEvent::end_element())?;
                        }
                        Tokenizer::Chunker(len) => {
                            let len_str = len.to_string();
                            writer.write(XmlEvent::start_element("ChunkTokenizer").attr("len", &len_str))?;
                            writer.write(XmlEvent::end_element())?;
                        }
                        Tokenizer::Ssp { ranks } => {
                            writer.write(XmlEvent::start_element("SspTokenizer"))?;

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
                                        XmlEvent::start_element("Class").attr("rank", &(rank + 1).to_string()),
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
                    writer.write(XmlEvent::start_element("Reject"))?;
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
                let mut ev = XmlEvent::start_element("Match");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }
                writer.write(ev)?;
                base.write_xml(writer, indent + 2)?;

                for (regex, config) in cases {
                    writer.write(XmlEvent::start_element("Case").attr("expr", &regex.as_str()))?;
                    config.write_xml(writer, indent + 4)?;
                    writer.write(XmlEvent::end_element())?;
                }

                if let Some(default) = default {
                    writer.write(XmlEvent::start_element("Default"))?;
                    default.write_xml(writer, indent + 2)?;
                    writer.write(XmlEvent::end_element())?;
                }

                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Numberer { id, style, min, max } => {
                let mut ev = XmlEvent::start_element("Number");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }
                writer.write(
                    ev.attr("style", &format!("{:?}", style))
                        .attr("min", &min.to_string())
                        .attr("max", &max.to_string()),
                )?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Optional {
                id,
                generator,
                probability,
            } => {
                let mut ev = XmlEvent::start_element("Option");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }

                writer.write(ev.attr("probability", &format!("{}", probability)))?;
                generator.write_xml(writer, indent + 2)?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Repeater {
                id,
                generator,
                min,
                max,
            } => {
                let mut ev = XmlEvent::start_element("Repeat");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }

                writer.write(ev.attr("min", &min.to_string()).attr("max", &max.to_string()))?;
                generator.write_xml(writer, indent + 2)?;
                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Switcher { id, subparts } => {
                let mut ev = XmlEvent::start_element("Switch");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
                }
                writer.write(ev)?;

                for subpart in subparts {
                    subpart.write_xml(writer, indent + 2)?;
                }

                writer.write(XmlEvent::end_element())?;
            }

            GeneratorConfig::Words { id, words } => {
                let mut ev = XmlEvent::start_element("Words");
                if let Some(id) = &id {
                    ev = ev.attr("id", id);
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
