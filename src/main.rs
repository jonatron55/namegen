mod generator;
mod styles;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Error as IoError, Read},
    path::PathBuf,
    process::ExitCode,
};

use anstream::eprintln;
use clap::Parser;
use rand::{Rng, SeedableRng, rngs::StdRng};
use xml::ParserConfig as XmlParserConfig;

use crate::styles::{ERROR, PATH, WARN};

const DEFAULT_CONFIG: &[u8] = include_bytes!("builtin/default.xml");
const THING_CONFIG: &[u8] = include_bytes!("builtin/thing.xml");

/// Generates random names from a given configuration.
#[derive(Parser)]
#[clap(about, long_about, version, author)]
struct Args {
    /// Path to generator configuration.
    ///
    /// This file may be either a plain text file or an XML configuration. The
    /// type will be inferred from the file extension if possible, or by the
    /// presence of an XML signature in the file contents.
    ///
    /// Plain text files will create a Markov generator trained on the
    /// whitespace-separated words in the file. XML files should follow the
    /// format described in the README and can be used to create more complex
    /// generators with multiple components.
    ///
    /// Built-in configurations are also available. The following names can be
    /// used to reference them:
    ///
    /// - `default`: A configuration that generates amusing person names.
    /// - `thing`: A configuration that generates amusing names for objects or
    ///   concepts.
    ///
    /// If a path is not provided, the `default` built-in configuration will be
    /// used. To use a configuration file that has the same name as a built-in,
    /// prefix it with `./` or another path component.
    #[arg(value_name = "FILE")]
    config: Option<PathBuf>,

    /// Number of names to generate.
    #[arg(long, short = 'n', default_value_t = 1)]
    count: usize,

    /// Analyze the given config file without generating names.
    ///
    /// This will output statistics about the Markov chain frequencies and
    /// combinatorial counts for the given config.
    #[arg(long, short, conflicts_with = "count")]
    analyze: bool,

    /// Enable verbose output in analysis mode.
    ///
    /// This will include the complete frequency table in the '--analyze'
    /// output.
    #[arg(long, short)]
    verbose: bool,

    /// Exports an example configuration file to the specified path instead of
    /// generating names.
    #[arg(long, short, conflicts_with = "analyze", conflicts_with = "count")]
    export: bool,

    /// Random seed for name generation.
    #[arg(long, short)]
    seed: Option<u64>,
}

fn main() -> ExitCode {
    let builtins = HashMap::from([
        (PathBuf::from("default"), DEFAULT_CONFIG),
        (PathBuf::from("thing"), THING_CONFIG),
    ]);

    let args = Args::parse();

    if args.export {
        let Some(path) = args.config else {
            eprintln!("{ERROR}Error:{ERROR:#} --export requires a path argument");
            return ExitCode::FAILURE;
        };
        if let Err(err) = fs::write(&path, DEFAULT_CONFIG) {
            eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }

        return ExitCode::SUCCESS;
    }

    let path = args.config.unwrap_or("default".into());

    // We accept either plain text or an XML config file. We'll base out initial
    // guess on the file extension. If it's not clear from the extension, we'll
    // peek at the start of the file later for an XML signature.
    let mut is_xml = match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("xml") => Some(true),
        Some(ext) if ext.eq_ignore_ascii_case("txt") => Some(false),
        _ => None,
    };

    let buffer: Box<dyn Read> = if let Some(builtin) = builtins.get(&path) {
        if path.exists() {
            eprintln!(
                "{WARN}Warning:{WARN:#} {PATH}{}{PATH:#} is a built-in configuration. To use the file with the same name, prefix it with ./ or another path component.",
                path.display()
            );
        }

        is_xml = Some(true);
        Box::new(*builtin)
    } else {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
                return ExitCode::FAILURE;
            }
        };
        Box::new(file)
    };

    let mut reader = BufReader::new(buffer);

    let is_xml = is_xml.map(Ok).unwrap_or_else(|| {
        let peek = reader.fill_buf()?;
        let prefix = &peek[..peek.len().min(16)];
        let prefix = String::from_utf8_lossy(prefix);
        let prefix = prefix.trim_start_matches('\u{feff}').trim_start();
        Ok::<bool, IoError>(prefix.starts_with("<?xml") || prefix.starts_with("<NameGen>"))
    });

    let is_xml = match is_xml {
        Ok(is_xml) => is_xml,
        Err(err) => {
            eprintln!("{ERROR}{}:{ERROR:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }
    };

    let generator = if is_xml {
        let mut xml = XmlParserConfig::new()
            .trim_whitespace(true)
            .whitespace_to_characters(true)
            .ignore_comments(true)
            .create_reader(reader);

        match generator::from_xml(&mut xml) {
            Ok(generator) => generator,
            Err(err) => {
                if let Some(position) = err.position() {
                    eprintln!(
                        "{ERROR}Error:{ERROR:#} {PATH}{}:{}:{PATH:#} {}",
                        path.display(),
                        position,
                        err
                    );
                } else {
                    eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
                }
                return ExitCode::FAILURE;
            }
        }
    } else {
        let mut text = String::new();
        if let Err(err) = reader.read_to_string(&mut text) {
            eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }

        generator::from_text(&text)
    };

    let mut rand: Box<dyn Rng> = match args.seed {
        Some(seed) => Box::new(StdRng::seed_from_u64(seed)),
        None => Box::new(rand::rng()),
    };

    if args.analyze {
        generator.analyze(args.verbose, 0);
        ExitCode::SUCCESS
    } else {
        for _ in 0..args.count {
            match generator.generate(&mut rand) {
                Ok(names) => {
                    for name in names {
                        print!("{name}");
                    }
                    println!();
                }
                Err(err) => {
                    eprintln!("{ERROR}Error:{ERROR:#} {err}");
                    return ExitCode::FAILURE;
                }
            }
        }

        ExitCode::SUCCESS
    }
}
