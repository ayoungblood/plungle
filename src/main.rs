// src/main.rs

use std::error::Error;
use std::path::Path;
use structopt::StructOpt;

mod frequency;
mod power;
mod radios;
mod structures;

// plungle - Radio codeplug conversion tool
// Usage: plungle [options] <radio> <input>
// Author: Akira Youngblood 2024
#[derive(StructOpt, Debug)]
#[structopt(name = "plungle", about = "Radio codeplug conversion tool")]
struct Opt {
    /// Radio model
    #[structopt(name = "radio")]
    radio: String,

    /// Input path
    #[structopt(name = "input", parse(from_os_str))]
    input: std::path::PathBuf,

    /// Dump option
    #[structopt(short = "D", long = "dump")]
    dump: Option<String>,

    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
}

// debug print
macro_rules! dprintln {
    ($opt_verbose:expr, $message_verbose:expr, $($arg:tt)*) => {
        if $opt_verbose >= $message_verbose {
            eprintln!($($arg)*);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    // all output except the actual codeplug data should go to stderr
    dprintln!(opt.verbose, 1, "Welcome to the plungle, we got fun and games");
    dprintln!(opt.verbose, 3, "{:?}", opt);

    // print the radio model
    dprintln!(opt.verbose, 1, "Radio model: {}", opt.radio);

    // input can be a file or a directory, check if it it exists
    if !Path::new(&opt.input).exists() {
        eprintln!("Input path does not exist: {}", opt.input.display());
        std::process::exit(1);
    }

    dprintln!(opt.verbose, 1, "Input path: {}", opt.input.display());

    let input_is_yaml = opt.input.is_file() && opt.input.extension().unwrap_or_default() == "yaml";

    if input_is_yaml { // input is a YAML file, we are generating a codeplug export
        dprintln!(opt.verbose, 1, "Parsing YAML file...");
        let yaml = std::fs::read_to_string(&opt.input)?;
        // let codeplug: structures::Codeplug = serde_yaml::from_str(&yaml)?;
        dprintln!(opt.verbose, 3, "{:?}", yaml);
    } else { // input is not YAML, we are parsing a codeplug export
        dprintln!(opt.verbose, 1, "Parsing codeplug export...");
        let codeplug = radios::parse(&opt.radio, &opt.input)?;
        dprintln!(opt.verbose, 3, "{:?}", codeplug);
    }

    // print the dump option
    if let Some(dump) = &opt.dump {
        println!("Dump option: {}", dump);
    }

    eprintln!("Completed with {} errors, {} warnings. Have a nice day!", 0, 0);
    Ok(())
}
