use std::{collections::HashMap, io::Write};

use itertools::Itertools;
use xml::{EventWriter as XmlWriter, writer::XmlEvent};

use crate::{
    config::{GeneratorConfig, WriteError, write_indented_lines},
    generator::{Generator, Markov, Tokenizer},
};

pub struct MarkovConfig {
    data: Vec<String>,
    target_len: Option<usize>,
    cutoff_len: Option<usize>,
    reject: Vec<String>,
    uniform: bool,
    reject_training: bool,
    tokenizer: Tokenizer,
}

impl MarkovConfig {
    pub fn new(
        data: Vec<String>,
        target_len: Option<usize>,
        cutoff_len: Option<usize>,
        reject: Vec<String>,
        reject_training: bool,
        uniform: bool,
        tokenizer: Tokenizer,
    ) -> Self {
        Self {
            data,
            target_len,
            cutoff_len,
            reject,
            uniform,
            reject_training,
            tokenizer,
        }
    }
}

impl GeneratorConfig for MarkovConfig {
    fn into_generator(mut self: Box<Self>) -> Box<dyn Generator> {
        if self.reject_training {
            self.reject.extend_from_slice(&self.data);
            self.reject.dedup();
        }

        Box::new(Markov::train(
            &self.data,
            self.target_len,
            self.cutoff_len,
            self.reject,
            &self.tokenizer,
            self.uniform,
        ))
    }

    fn write_xml(
        self: Box<Self>,
        writer: &mut XmlWriter<&mut Box<dyn Write>>,
        indent: usize,
    ) -> Result<(), WriteError> {
        let mut builder = XmlEvent::start_element("Markov");

        let target_len_str: String;
        let cutoff_len_str: String;

        if let Some(target_len) = self.target_len {
            target_len_str = target_len.to_string();
            builder = builder.attr("target_len", &target_len_str);
        }

        if let Some(cutoff_len) = self.cutoff_len {
            cutoff_len_str = cutoff_len.to_string();
            builder = builder.attr("cutoff_len", &cutoff_len_str);
        }

        if self.uniform {
            builder = builder.attr("uniform", "true");
        }

        if self.reject_training {
            builder = builder.attr("reject_training", "true");
        }

        writer.write(builder)?;

        if self.tokenizer != Tokenizer::default_ssp() {
            match self.tokenizer {
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
                            writer.write(XmlEvent::start_element("Class").attr("rank", &(rank + 1).to_string()))?;
                            writer.write(XmlEvent::characters(&class))?;
                            writer.write(XmlEvent::end_element())?;
                        }
                    }

                    writer.write(XmlEvent::end_element())?;
                }
            }
        }

        if self.reject.len() > 0 {
            writer.write(XmlEvent::start_element("Reject"))?;
            write_indented_lines(self.reject, indent + 4, writer)?;
            writer.write(XmlEvent::end_element())?;
        }

        write_indented_lines(self.data, indent + 2, writer)?;

        writer.write(XmlEvent::end_element())?;

        Ok(())
    }
}
