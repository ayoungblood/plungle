// src/main.rs

use std::error::Error;
//use std::path::Path;
use structopt::StructOpt;

// mod frequency;
// mod power;
// mod radios;
// mod structures;

// plungle - Radio codeplug conversion tool
// Usage: plungle [options] <operation> [<args>]
// Author: Akira Youngblood 2024
#[derive(StructOpt, Debug)]
#[structopt(name = "plungle", about = "Radio codeplug conversion tool")]
struct Opt {
    /// Operation
    #[structopt(name = "operation", required = true)]
    operation: String,

    /// Radio model
    #[structopt(name = "radio")]
    radio: Option<String>,

    /// Input path
    #[structopt(name = "input", parse(from_os_str))]
    input: Option<std::path::PathBuf>,

    /// Output path
    #[structopt(name = "output", parse(from_os_str))]
    output: Option<std::path::PathBuf>,

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

    // parse the operation
    dprintln!(opt.verbose, 1, "Operation: {}", opt.operation);
    if let Some(radio) = &opt.radio {
        dprintln!(opt.verbose, 1, "Radio model: {}", radio);
    }
    if opt.operation == "read" || opt.operation == "r" {
        eprintln!("Reading codeplug...");
    } else if opt.operation == "write" || opt.operation == "w" {
        eprintln!("Writing codeplug...");
    } else if opt.operation == "validate" || opt.operation == "v" {
        eprintln!("Validating codeplug...");
    } else if opt.operation == "filter" || opt.operation == "f" {
        eprintln!("Filtering codeplug...");
    } else {
        eprintln!("Invalid operation: {}", opt.operation);
        std::process::exit(1);
    }

    // // print the radio model
    // dprintln!(opt.verbose, 1, "Radio model: {}", opt.radio);

    // // input can be a file or a directory, check if it it exists
    // if !Path::new(&opt.input).exists() {
    //     eprintln!("Input path does not exist: {}", opt.input.display());
    //     std::process::exit(1);
    // }

    // dprintln!(opt.verbose, 1, "Input path: {}", opt.input.display());

    // let input_is_yaml = opt.input.is_file() && opt.input.extension().unwrap_or_default() == "yaml";

    // if input_is_yaml { // input is a YAML file, we are generating a codeplug export
    //     dprintln!(opt.verbose, 1, "Parsing YAML file...");
    //     let yaml = std::fs::read_to_string(&opt.input)?;
    //     // let codeplug: structures::Codeplug = serde_yaml::from_str(&yaml)?;
    //     dprintln!(opt.verbose, 3, "{:?}", yaml);
    // } else { // input is not YAML, we are parsing a codeplug export
    //     dprintln!(opt.verbose, 1, "Parsing codeplug export...");
    //     let codeplug = radios::parse(&opt.radio, &opt.input)?;
    //     dprintln!(opt.verbose, 3, "{:?}", codeplug);
    // }

    eprintln!("Completed with {} errors, {} warnings. Have a nice day!", 0, 0);
    Ok(())
}
