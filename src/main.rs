// src/main.rs
// plungle - Radio codeplug conversion tool
// Author: Akira Youngblood 2024

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::error::Error;
use helpers::*;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use crate::Dest::{Stdout, Stderr};

mod helpers;
mod radios;
mod structures;
mod validate;
mod bandplan;
mod printer;

#[derive(Debug, Parser)]
#[clap(version, author, about = "Codeplug conversion tool")]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global=true)]
    verbose: u8,

    /// Color output
    #[arg(short, long, default_value_t, value_enum, global=true)]
    color: clap::ColorChoice,

    /// Intermediary format
    #[arg(short = 'F', long, default_value_t, value_enum, global=true)]
    format: helpers::Format,

    /// Filter
    #[arg(short, long, global=true)]
    filter: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
enum Commands {
    /// Parse radio-specific codeplug into an intermediary format
    Parse {
        /// Radio model
        model: String,
        /// Input path
        input: PathBuf,
        /// Output path (stdout is used if not specified)
        output: Option<PathBuf>,
    },
    /// Generate a radio-specific codeplug from an intermediary format
    Generate {
        /// Radio model
        model: String,
        /// Input path
        input: PathBuf,
        /// Output path
        output: PathBuf,
    },
    /// Merge codeplugs
    Merge {
        /// Input paths
        inputs: Vec<PathBuf>,
    },
}

// @TODO re-implement filtering and move to a separate module
fn filter_codeplug(codeplug: &structures::Codeplug, _opt: &Opt) -> Result<structures::Codeplug, Box<dyn Error>> {
    return Ok(codeplug.clone());
}

fn read_codeplug(input_path: &PathBuf, opt: &Opt) -> Result<structures::Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    // if we recognize the file extension, use it to determine the file format
    // otherwise, use --format (which defaults to JSON)
    let format = match input_path.extension() {
        Some(ext) => {
            match ext.to_str().unwrap() {
                "json" => helpers::Format::Json,
                "toml" => helpers::Format::Toml,
                _ => opt.format.clone(),
            }
        }
        None => opt.format.clone(),
    };
    // read the codeplug
    let codeplug: structures::Codeplug;
    if format == helpers::Format::Json {
        uprintln!(opt, Stderr, Color::Green, None, "Reading codeplug as JSON from: {:?}", input_path);
        codeplug = serde_json::from_str(&std::fs::read_to_string(input_path)?)?;
    } else if format == helpers::Format::Toml {
        uprintln!(opt, Stderr, Color::Green, None, "Reading codeplug as TOML from: {:?}", input_path);
        codeplug = toml::from_str(&std::fs::read_to_string(input_path)?)?;
    } else {
        uprintln!(opt, Stderr, Color::Red, None, "Unsupported codeplug format");
        return Err("Unsupported codeplug format".into());
    }

    Ok(codeplug)
}

fn write_codeplug(output_path: &Option<PathBuf>, codeplug: &structures::Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    // if we recognize the file extension, use it to determine the file format
    // otherwise, use --format (which defaults to JSON)
    let format = match output_path {
        Some(path) => {
            match path.extension() {
                Some(ext) => {
                    match ext.to_str().unwrap() {
                        "json" => helpers::Format::Json,
                        "toml" => helpers::Format::Toml,
                        "txt" => helpers::Format::Text,
                        _ => opt.format.clone(),
                    }
                }
                None => opt.format.clone(),
            }
        }
        None => opt.format.clone(),
    };
    // serialize the codeplug to a string
    let file_str = match format {
        helpers::Format::Json => serde_json::to_string_pretty(codeplug)?,
        helpers::Format::Toml => toml::to_string_pretty(codeplug)?,
        helpers::Format::Text => printer::pretty(opt, codeplug)?,
    };

    // write to file or stdout
    if output_path.is_none() {
        uprintln!(opt, Stderr, Color::Green, None, "Writing codeplug to stdout");
        uprintln!(opt, Stdout, None, None, "{}", file_str);
    } else {
        uprintln!(opt, Stderr, Color::Green, None, "Writing codeplug to {:?}", output_path.as_ref().unwrap());
        std::fs::write(output_path.as_ref().unwrap(), file_str)?;
    }

    uprintln!(opt, Stderr, Color::Cyan, None, "Codeplug has {} channels, {} zones, {} talkgroups, {} talkgroup lists",
        codeplug.channels.len(), codeplug.zones.len(), codeplug.talkgroups.len(), codeplug.talkgroup_lists.len());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::parse();
    // all output except the actual codeplug data should go to stderr
    uprintln!(opt, Stderr, Color::Green, 1, "Welcome to the plungle, we got fun and games!");
    uprintln!(opt, Stderr, None, 3, "{:?}", opt);

    match &opt.command {
        Some(Commands::Parse { model, input, output }) => {
            // parse codeplug
            let mut codeplug = radios::parse_codeplug(model, input, &opt)?;
            // filter codeplug
            codeplug = filter_codeplug(&codeplug, &opt)?;
            // validate codeplug
            radios::validate_codeplug(&codeplug, &model, &opt)?;
            // write intermediary file
            write_codeplug(&output, &codeplug, &opt)?;
        }
        Some(Commands::Generate { model, input, output }) => {
            // read intermediary file
            let mut codeplug = read_codeplug(&input, &opt)?;
            // filter codeplug
            codeplug = filter_codeplug(&codeplug, &opt)?;
            // validate codeplug
            radios::validate_codeplug(&codeplug, &model, &opt)?;
            // generate codeplug
            radios::generate_codeplug(&codeplug, &model, &output, &opt)?;
        }
        Some(Commands::Merge { inputs: _ }) => {
            // @TODO implement merge
            uprintln!(opt, Stderr, Color::Red, None, "merge - Operation unsupported!");
        }
        None => { // this should never happen because of arg_required_else_help
            uprintln!(opt, Stderr, Color::Red, None, "No command specified");
        }
    }
    Ok(())
}
