// src/main.rs
// plungle - Radio codeplug conversion tool
// Author: Akira Youngblood 2024

mod frequency;
mod helpers;
mod radios;
mod structures;
mod validate;
mod bandplan;
mod printer;
mod merge;
mod filter;
mod theme;

use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::error::Error;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use helpers::*;
use crate::Dest::{Stdout, Stderr};
use crate::theme::THEME;

lazy_static! {
    static ref VERSION: String = get_version_fancy();
}

#[derive(Debug, Parser)]
#[command(version = VERSION.as_str())]
#[command(author = "Akira Youngblood")]
#[command(about = "Codeplug conversion tool for radio configuration")]
#[command(help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]

struct Opt {
    /// Color output
    #[arg(short, long, default_value_t, value_enum, global=true)]
    color: clap::ColorChoice,

    /// Filter
    #[arg(short, long, global=true)]
    filter: Option<Vec<String>>,

    /// Intermediary format
    #[arg(short = 'F', long, default_value_t, value_enum, global=true)]
    format: helpers::Format,

    /// Quiet output
    #[arg(short, long, action = clap::ArgAction::SetTrue, global=true)]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global=true)]
    verbose: u8,

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

pub fn get_version_fancy() -> String {
    if env!("GIT_AVAILABLE") == "true" {
        let base_version = env!("CARGO_PKG_VERSION");
        let git_sha = env!("GIT_SHA");
        let git_branch = env!("GIT_BRANCH");
        format!("{} ({} {})", base_version, git_sha, git_branch)
    } else {
        format!("{}", env!("CARGO_PKG_VERSION"))
    }
}

fn read_codeplug(opt: &Opt, input_path: &PathBuf) -> Result<structures::Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
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
        if !opt.quiet { uprintln!(opt, Stderr, THEME.info, None, "Reading codeplug as JSON from: {:?}", input_path); }
        codeplug = serde_json::from_str(&std::fs::read_to_string(input_path)?)?;
    } else if format == helpers::Format::Toml {
        if !opt.quiet { uprintln!(opt, Stderr, THEME.info, None, "Reading codeplug as TOML from: {:?}", input_path); }
        codeplug = toml::from_str(&std::fs::read_to_string(input_path)?)?;
    } else {
        uprintln!(opt, Stderr, THEME.err, None, "Unsupported codeplug format");
        return Err("Unsupported codeplug format".into());
    }

    Ok(codeplug)
}

fn write_codeplug(opt: &Opt, output_path: &Option<PathBuf>, codeplug: &structures::Codeplug) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    // if --format is Default, and we recognize the file extension, use it to determine the file format
    // otherwise, use --format
    let format = match opt.format {
        helpers::Format::Default => {
            match output_path {
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
            }
        },
        _ => opt.format.clone(),
    };
    // serialize the codeplug to a string
    let file_str = match format {
        helpers::Format::Json => serde_json::to_string_pretty(codeplug)?,
        helpers::Format::Toml => toml::to_string_pretty(codeplug)?,
        helpers::Format::Text => printer::pretty(opt, codeplug)?,
        helpers::Format::Default => printer::pretty(opt, codeplug)?,
    };

    // write to file or stdout
    if output_path.is_none() {
        if !opt.quiet { uprintln!(opt, Stderr, THEME.info, None, "Writing codeplug to stdout (--format={})", format); }
        uprintln!(opt, Stdout, None, None, "{}", file_str);
    } else {
        if !opt.quiet { uprintln!(opt, Stderr, THEME.info, None, "Writing codeplug to {:?} (--format={})", output_path.as_ref().unwrap(), format); }
        std::fs::write(output_path.as_ref().unwrap(), file_str)?;
    }

    if !opt.quiet { uprintln!(opt, Stderr, Color::Cyan, None, "Codeplug has {} channels, {} zones, {} talkgroups, {} talkgroup lists",
        codeplug.channels.len(), codeplug.zones.len(), codeplug.talkgroups.len(), codeplug.talkgroup_lists.len()); }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::parse();
    // all output except the actual codeplug data should go to stderr
    uprintln!(opt, Stderr, THEME.info, 1, "Welcome to the plungle, we got fun and games!");
    uprintln!(opt, Stderr, None, 3, "{:?}", opt);

    match &opt.command {
        Some(Commands::Parse { model, input, output }) => {
            // parse codeplug
            let mut codeplug = radios::parse_codeplug(&opt, model, input)?;
            // filter codeplug
            codeplug = filter::filter_codeplug(&opt, &codeplug, &opt.filter)?;
            // validate codeplug
            validate::validate_codeplug(&opt, &codeplug, Some(&model))?;
            // write intermediary file
            write_codeplug(&opt, &output, &codeplug)?;
        }
        Some(Commands::Generate { model, input, output }) => {
            // read intermediary file
            let mut codeplug = read_codeplug(&opt, &input)?;
            // filter codeplug
            codeplug = filter::filter_codeplug(&opt, &codeplug, &opt.filter)?;
            // validate codeplug
            validate::validate_codeplug(&opt, &codeplug, Some(&model))?;
            // generate codeplug
            radios::generate_codeplug(&opt, &codeplug, &model, &output)?;
        }
        Some(Commands::Merge { inputs }) => {
            // merge codeplugs
            let mut codeplug = merge::merge_codeplug(&opt, &inputs)?;
            // filter codeplug
            codeplug = filter::filter_codeplug(&opt, &codeplug, &opt.filter)?;
            // validate codeplug (without a model, for now)
            validate::validate_codeplug(&opt, &codeplug, None)?;
            // write intermediary file
            write_codeplug(&opt, &None, &codeplug)?; // @TODO FIXME
        }
        None => { // this should never happen because of arg_required_else_help
            uprintln!(opt, Stderr, THEME.err, None, "No command specified");
        }
    }
    Ok(())
}
