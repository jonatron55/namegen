mod generator;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use rand::{Rng, SeedableRng, rngs::StdRng};
use xml::ParserConfig as XmlParserConfig;

/// Generates random names from a given configuration.
#[derive(Parser)]
#[clap(about, long_about, version, author)]
struct Args {
    /// Path to XML config file.
    ///
    /// If not provided, the default config embedded in the binary will be used.
    config: Option<PathBuf>,

    /// Number of names to generate.
    #[arg(long, short = 'n', default_value_t = 1)]
    count: usize,

    /// Analyze the given config file without generating names.
    ///
    /// This will output statistics about the Markov chain frequencies and
    /// combinatorial counts for the given config.
    #[arg(long, short)]
    analyze: bool,

    /// Random seed for name generation.
    #[arg(long, short)]
    seed: Option<u64>,
}

fn main() {
    let args = Args::parse();
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

    let generator = generator::from_xml(&mut xml).expect("Failed to parse config file");

    let mut rand: Box<dyn Rng> = match args.seed {
        Some(seed) => Box::new(StdRng::seed_from_u64(seed)),
        None => Box::new(rand::rng()),
    };

    if args.analyze {
        generator.print_analysis(0);
    } else {
        for _ in 0..args.count {
            for name in generator.generate(&mut rand) {
                print!("{name}");
            }
            println!();
        }
    }
}
