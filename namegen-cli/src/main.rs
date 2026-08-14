mod styles;

use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Error as IoError, Read, Write},
    path::PathBuf,
    process::ExitCode,
};

use anstream::eprintln;
use clap::Parser;
use libnamegen::config::{ConfigSourceType, GeneratorConfig, IntoGenerator, WriteXml};
use rand::{rngs::StdRng, Rng, SeedableRng};
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

    /// Transliterates characters in the generated names to their closest
    /// runic equivalent.
    ///
    /// This flag maps characters to their corresponding runes in the Anglo-
    /// Saxon Futhorc. Characters that do not have a clear Futhorc equivalent
    /// remain unchanged.
    #[arg(long, short, alias = "futhark", alias = "runes", conflicts_with = "ascii")]
    futhorc: bool,

    /// Exports an example configuration file to the specified path instead of
    /// generating names.
    ///
    /// An XML schema file will also be exported to the same directory.
    #[arg(long, short, conflicts_with = "count", conflicts_with = "beautify")]
    export: bool,

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
    let (buffer, is_xml): (Box<dyn Read>, Option<bool>) = if let Some(path) = args.config.as_deref() {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
                return ExitCode::FAILURE;
            }
        };

        let is_xml = match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("xml") => Some(true),
            Some(ext) if ext.eq_ignore_ascii_case("txt") => Some(false),
            _ => None,
        };

        (Box::new(file), is_xml)
    } else {
        (Box::new(DEFAULT_CONFIG), Some(true))
    };

    let mut reader = BufReader::new(buffer);

    let is_xml = is_xml.map(Ok).unwrap_or_else(|| {
        let peek = reader.fill_buf()?;
        let prefix = &peek[..peek.len().min(16)];
        let prefix = String::from_utf8_lossy(prefix);
        let prefix = prefix.trim_start_matches('\u{feff}').trim_start();
        Ok::<bool, IoError>(prefix.starts_with("<?xml") || prefix.starts_with("<NameGen>"))
    });

    let path = args.config.unwrap_or_else(|| PathBuf::from("<default>"));

    let is_xml = match is_xml {
        Ok(is_xml) => is_xml,
        Err(err) => {
            eprintln!("{ERROR}{}:{ERROR:#} {}", path.display(), err);
            return ExitCode::FAILURE;
        }
    };

    let config = match GeneratorConfig::read(
        reader,
        if is_xml {
            ConfigSourceType::Xml
        } else {
            ConfigSourceType::PlainText
        },
    ) {
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

        let output = match OpenOptions::new().write(true).create(true).open(&tmp) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("{ERROR}Error:{ERROR:#} {PATH}{}:{PATH:#} {}", path.display(), err);
                return ExitCode::FAILURE;
            }
        };

        let mut writer: Box<dyn Write> = Box::new(output);
        let mut writer = XmlEmitterConfig::new()
            .perform_indent(true)
            .line_separator("\n")
            .pad_self_closing(true)
            .indent_string("  ")
            .create_writer(&mut writer);

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

    for _ in 0..args.count {
        match generator.generate(&mut rand, &constraints) {
            Ok(names) => {
                for name in names {
                    if args.ascii {
                        let ascii_name = translit::to_ascii(&name);
                        print!("{ascii_name}");
                    } else if args.futhorc {
                        let futhark_name = translit::to_futhorc(&name);
                        print!("{futhark_name}");
                    } else {
                        print!("{name}");
                    }
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
