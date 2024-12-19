use std::error::Error;
use std::path::Path;
use structopt::StructOpt;

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

    // print the radio model
    dprintln!(opt.verbose, 1, "Radio model: {}", opt.radio);

    // input can be a file or a directory, check if it it exists
    if !Path::new(&opt.input).exists() {
        eprintln!("Input path does not exist: {}", opt.input.display());
        std::process::exit(1);
    }

    dprintln!(opt.verbose, 1, "Input path: {}", opt.input.display());

    let input_is_dir = opt.input.is_dir();

    // determine operation
    if input_is_dir && opt.radio == "anytone_x78" {
        eprintln!("Converting Anytone x78 CSV export to YAML");
    } else {
        eprintln!("Cannot infer operation for radio model {} and input path {}", opt.radio, opt.input.display());
        std::process::exit(1);
    }

    // print the dump option
    if let Some(dump) = &opt.dump {
        println!("Dump option: {}", dump);
    }

    eprintln!("Completed with {} errors, {} warnings. Have a nice day!", 0, 0);
    Ok(())
}
