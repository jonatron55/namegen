mod concatter;
mod markov;
mod namegen;

use std::fs::File;
use std::io::BufReader;

use clap::Parser;
use namegen::config::Config;
use namegen::NameGen;
use rand::thread_rng;
use xml::{EventReader as XmlReader, ParserConfig as XmlParserConfig};

#[derive(Parser)]
struct Cli {
    config: String,

    #[arg(long, short = 'n', default_value = "10")]
    count: Option<usize>,
}

fn main() {
    let args = Cli::parse();
    let file = File::open(&args.config).expect("Failed to open config file");
    let reader: BufReader<File> = BufReader::new(file);
    let mut xml = XmlParserConfig::new()
        .trim_whitespace(true)
        .whitespace_to_characters(true)
        .ignore_comments(true)
        .create_reader(reader);

    let config = Config::from_xml(&mut xml)
        .expect("Failed to parse config file");

    let namegen = NameGen::from_config(config);

    let mut rng= thread_rng();
    for _ in 0..args.count.unwrap_or(10) {
        println!("{}", namegen.generate(&mut rng));
    }
}
