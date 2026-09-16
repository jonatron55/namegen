mod styles;

use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufReader, Read},
    path::PathBuf,
    process::ExitCode,
};

use anstream::eprintln;
use clap::Parser;
use libnamegen::config::{ConfigSourceType, GeneratorConfig, IntoGenerator, WriteXml};
use rand::{rngs::StdRng, Rng, SeedableRng};
use translit::Casing;
use xml::EmitterConfig as XmlEmitterConfig;

use crate::styles::{ERROR, PATH};

const DEFAULT_CONFIG: &[u8] = include_bytes!("../../configs/silly.xml");
const SCHEMA: &[u8] = include_bytes!("../../configs/namegen.xsd");

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
    /// If a path is not provided, a default built-in configuration will be
    /// used.
    #[arg(value_name = "FILE")]
    config: Option<PathBuf>,

    /// Number of names to generate.
    #[arg(long, short = 'n', default_value_t = 1)]
    count: usize,

    /// Constrain the output of a particular generator with the given ID.
    /// Behaviour differs based on the generator type.
    ///
    /// This allows you to steer the generation process by providing specific
    /// constraints for certain generators. This option can be used multiple
    /// times to provide constraints for multiple generators. It should be
    /// provided in the format `<id>:<constraint>`.
    #[arg(long, short = 'C', conflicts_with = "export", conflicts_with = "beautify")]
    constrain: Vec<String>,

    /// Random seed for name generation.
    #[arg(long, short, conflicts_with = "export", conflicts_with = "beautify")]
    seed: Option<u64>,

    /// Replaces the provided configuration file with a beautified version of
    /// the same configuration and produces no other output.
    ///
    /// The output will be formatted with indentation and line breaks. Elements
    /// such as <Markov> and <Words> will be sorted and deduplicated. Plain text
    /// configurations will be converted to XML.
    #[arg(long, short, conflicts_with = "count", conflicts_with = "export")]
    beautify: bool,

    /// Converts non-ASCII characters in the generated names to their closest
    /// ASCII equivalent.
    ///
    /// This flag maps accented characters to their unaccented counterparts, and
    /// replaces other non-ASCII characters with their closest approximations
    /// (for example, "ð" becomes "th", and "ß" becomes "ss"). Characters that
    /// do not have a clear ASCII equivalent will be replaced by '?'.
    #[arg(long, short)]
    ascii: bool,

    /// Converts non-ASCII characters and renders the names in snake_case.
    ///
    /// This will replace non-ASCII characters in the same way as the 'ascii'
    /// flag, and then convert all letters to lowercase and replace whitespace
    /// with '_'.
    #[arg(long, group = "transliteration")]
    snake: bool,

    /// Converts non-ASCII characters and renders the names in kebab-case.
    ///
    /// This will replace non-ASCII characters in the same way as the 'ascii'
    /// flag, and then convert them to lowercase and replace whitespace with
    /// '-'.
    #[arg(long, group = "transliteration")]
    kebab: bool,

    /// Converts non-ASCII characters and renders the names in camelCase.
    ///
    /// This will replace non-ASCII characters in the same way as the 'ascii'
    /// flag, and then remove whitespace while capitalizing the first letter of
    /// every word except the first.
    #[arg(long, group = "transliteration")]
    camel: bool,

    /// Converts non-ASCII characters and renders the names in PascalCase.
    ///
    /// This will replace non-ASCII characters in the same way as the 'ascii'
    /// flag, and then remove whitespace while capitalizing the first letter of
    /// every word.
    #[arg(long, group = "transliteration")]
    pascal: bool,

    /// Converts non-ASCII characters and renders the names in SCREAMING_CASE.
    ///
    /// This will replace non-ASCII characters in the same way as the 'ascii'
    /// flag, and then convert all letters to uppercase and replace whitespace
    /// with '_'.
    #[arg(long, alias = "scream", alias = "screaming-snake", group = "transliteration")]
    screaming: bool,

    /// Transliterates characters in the generated names to their closest
    /// runic equivalent.
    ///
    /// This flag maps characters to their corresponding runes in the Anglo-
    /// Saxon Futhorc. Characters that do not have a clear Futhorc equivalent
    /// remain unchanged.
    #[arg(long, alias = "futhark", alias = "runes", group = "transliteration")]
    futhorc: bool,

    /// Transliterates characters in the generated names to their closest
    /// Tengwar equivalent.
    ///
    /// This flag maps characters to their corresponding symbols in the Tengwar
    /// script. Characters that do not have a clear Tengwar equivalent remain
    /// unchanged.
    ///
    /// The output will use the ConScript Unicode Registry (CSUR) encoding for
    /// Tengwar (U+E000 to U+E07F).
    #[arg(long, alias = "tengwar", group = "transliteration")]
    tengwar: bool,

    /// Exports an example configuration file to the specified path instead of
    /// generating names.
    ///
    /// An XML schema file will also be exported to the same directory.
    #[arg(long, short, conflicts_with = "count", conflicts_with = "beautify")]
    export: bool,
}

fn main() -> ExitCode {
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

        println!("Exported '{}'", path.display());

        let schema_path = path.with_file_name("namegen.xsd");
        if let Err(err) = fs::write(&schema_path, SCHEMA) {
            eprintln!(
                "{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}",
                schema_path.display(),
                err
            );
            return ExitCode::FAILURE;
        }

        println!("Exported '{}'", schema_path.display());

        return ExitCode::SUCCESS;
    }

    // We accept either plain text or an XML config file. We'll base out initial
    // guess on the file extension. If it's not clear from the extension, we'll
    // peek at the start of the file later for an XML signature.
    let buffer: Box<dyn Read> = if let Some(path) = args.config.as_deref() {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
                return ExitCode::FAILURE;
            }
        };

        Box::new(file)
    } else {
        Box::new(DEFAULT_CONFIG)
    };

    let mut reader = BufReader::new(buffer);

    let source_type = if let Some(ref path) = args.config {
        match ConfigSourceType::guess(&path, &mut reader) {
            Ok(source_type) => source_type,
            Err(err) => {
                eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
                return ExitCode::FAILURE;
            }
        }
    } else {
        ConfigSourceType::Xml
    };

    let path = args.config.unwrap_or_else(|| PathBuf::from("<default>"));

    let config = match GeneratorConfig::read(reader, source_type) {
        Ok(config) => config,
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
    };

    if args.beautify {
        let tmp = path.with_extension("tmp");

        let mut output = match OpenOptions::new().write(true).create(true).open(&tmp) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
                return ExitCode::FAILURE;
            }
        };

        let mut writer = XmlEmitterConfig::new()
            .perform_indent(true)
            .line_separator("\n")
            .pad_self_closing(true)
            .indent_string("  ")
            .create_writer(&mut output);

        if let Err(err) = config.write_xml_root(&mut writer) {
            eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }

        let bak = path.with_extension("bak");

        _ = fs::remove_file(&bak);

        if let Err(err) = fs::rename(&path, &bak) {
            eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }

        if let Err(err) = fs::rename(&tmp, &path) {
            eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }

        if let Err(err) = fs::remove_file(&bak) {
            eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }

        return ExitCode::SUCCESS;
    }

    let generator = config.into_generator();

    let mut rand: Box<dyn Rng> = match args.seed {
        Some(seed) => Box::new(StdRng::seed_from_u64(seed)),
        None => Box::new(rand::rng()),
    };

    let constraints: HashMap<&str, &str> = args
        .constrain
        .iter()
        .filter_map(|constraint| {
            let mut parts = constraint.splitn(2, ':');
            let id = parts.next()?.trim();
            let value = parts.next()?.trim();
            Some((id, value))
        })
        .collect();

    let transliterator = if args.ascii {
        translit::to_ascii
    } else if args.snake {
        |s: &str| translit::to_ascii_with_casing(s, Casing::Snake)
    } else if args.kebab {
        |s: &str| translit::to_ascii_with_casing(s, Casing::Kebab)
    } else if args.camel {
        |s: &str| translit::to_ascii_with_casing(s, Casing::Camel)
    } else if args.pascal {
        |s: &str| translit::to_ascii_with_casing(s, Casing::Pascal)
    } else if args.screaming {
        |s: &str| translit::to_ascii_with_casing(s, Casing::Screaming)
    } else if args.futhorc {
        translit::to_futhorc
    } else if args.tengwar {
        translit::to_tengwar
    } else {
        |s: &str| s.to_string()
    };

    for _ in 0..args.count {
        match generator.generate(&mut rand, &constraints) {
            Ok(names) => {
                for name in names {
                    let name = transliterator(&name);
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
