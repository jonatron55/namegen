mod generator;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use xml::ParserConfig as XmlParserConfig;

#[derive(Parser)]
struct Cli {
    config: Option<PathBuf>,

    #[arg(long, short = 'n', default_value_t = 1)]
    count: usize,
}

fn main() {
    let args = Cli::parse();
    let reader: Box<dyn Read> = match args.config {
        Some(path) => {
            let file = File::open(&path).expect("Failed to open config file");
            Box::new(file)
        }
        None => {
            let default = include_bytes!("default.xml");
            Box::new(&default[..])
        }
    };

    let reader = BufReader::new(reader);
    let mut xml = XmlParserConfig::new()
        .trim_whitespace(true)
        .whitespace_to_characters(true)
        .ignore_comments(true)
        .create_reader(reader);

    let gen = generator::from_xml(&mut xml).expect("Failed to parse config file");

    let mut rand = rand::rng();
    for _ in 0..args.count {
        for name in gen.generate(&mut rand) {
            print!("{name}");
        }
        println!();
    }
}
