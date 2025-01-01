// src/main.rs

use std::error::Error;
use structopt::StructOpt;
use helpers::*;

mod helpers;
mod radios;
mod structures;

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

    /// Dump
    #[structopt(short, long)]
    dump: Option<String>,

    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,
}

fn dump(codeplug: &structures::Codeplug, opt: &Opt) -> Result<structures::Codeplug, Box<dyn Error>> {
    let mut new_codeplug = structures::Codeplug {
        channels: Vec::new(),
        zones: Vec::new(),
        talkgroups: Vec::new(),
        talkgroup_lists: Vec::new(),
        config: None,
    };
    // we are dumping everything
    if opt.dump.is_none() || opt.dump.as_ref().unwrap() == "all" {
        new_codeplug = codeplug.clone();
    } else {
        let dump = opt.dump.as_ref().unwrap();
        // split dump string into a vector
        let dump_vec: Vec<&str> = dump.split(',').collect();
        for dd in dump_vec {
            dprintln!(opt.verbose, 3, "Processing dump item: {}", dd);
            // if a dump argument starts with c, it applies to channels
            if dd.starts_with("c") { // channels
                if dd.contains("-") {
                    let range: Vec<&str> = dd.trim_start_matches('c').split('-').collect();
                    let start = range[0].parse::<usize>().unwrap();
                    let end = range[1].parse::<usize>().unwrap();
                    dprintln!(opt.verbose, 3, "Dump range: {}-{}", start, end);
                    for ii in start..=end {
                        // add channel by index to new codeplug
                        for cc in codeplug.channels.iter() {
                            if cc.index == ii as u32 {
                                new_codeplug.channels.push(cc.clone());
                            }
                        }
                    }

                } else {
                    let index = dd.trim_start_matches("c").parse::<usize>().unwrap();
                    dprintln!(opt.verbose, 3, "Dump index: {}", index);
                    // add channel by index to new codeplug
                    for cc in codeplug.channels.iter() {
                        if cc.index == index as u32 {
                            new_codeplug.channels.push(cc.clone());
                        }
                    }
                }
            } else {
                cprintln!(ANSI_C_YLW, "Unsupported dump item: {}", dd);
            }
        }
    }
    // dump to JSON (@TODO add support for YAML/TOML)
    let json = serde_json::to_string_pretty(&new_codeplug)?;
    println!("{}", json);
    eprintln!("Codeplug has {} channels, {} zones, {} talkgroups, {} talkgroup lists",
        new_codeplug.channels.len(), new_codeplug.zones.len(), new_codeplug.talkgroups.len(), new_codeplug.talkgroup_lists.len());
    return Ok(new_codeplug);
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    // all output except the actual codeplug data should go to stderr
    dprintln!(opt.verbose, 1, "Welcome to the plungle, we got fun and games!");
    dprintln!(opt.verbose, 3, "{:?}", opt);

    // parse the operation
    if opt.operation == "read" || opt.operation == "r" {
        // read() validates the radio model and input path
        let codeplug = radios::read_codeplug(&opt)?;
        // dump codeplug
        dump(&codeplug, &opt)?;
    } else if opt.operation == "write" || opt.operation == "w" {
        // make sure we have a radio model
        if opt.radio.is_none() {
            eprintln!("Radio model is required for operation: write");
            std::process::exit(1);
        }
        eprintln!("Writing codeplug...");
        eprintln!("Output path: {}", opt.output.as_ref().unwrap().display());
    } else if opt.operation == "validate" || opt.operation == "v" {
        eprintln!("Validating codeplug...");
    } else if opt.operation == "filter" || opt.operation == "f" {
        eprintln!("Filtering codeplug...");
    } else {
        eprintln!("Invalid operation: {}", opt.operation);
        std::process::exit(1);
    }

    eprintln!("Completed with {} errors, {} warnings. Have a nice day!", 0, 0);
    Ok(())
}
