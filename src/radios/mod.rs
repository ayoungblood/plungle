// src/radios/mod.rs

use std::error::Error;
use std::collections::HashMap;

use crate::Opt;
use crate::structures::Codeplug;
use crate::*;
use crate::validate::validate_generic;

mod anytone_x78;
mod opengd77_rt3s;

fn print_supported_radios_read() {
    eprintln!("Operation read supports the following radio models:");
    eprintln!("    anytone_x78 - Anytone AT-D878UV, etc");
}

pub fn read_codeplug(opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // validate the radio
    if opt.radio.is_none() {
        cprintln!(ANSI_C_RED, "Radio model is required for operation: read");
        return Err("Bad radio model".into());
    }
    let radio_model = opt.radio.as_ref().unwrap();
    // validate the input path
    if opt.input.is_none() {
        cprintln!(ANSI_C_RED, "Input path is required for operation: read");
        return Err("Bad input path".into());
    }

    // search for the radio model in the supported radios
    if "anytone_x78".contains(radio_model) {
        return anytone_x78::read(opt);
    } else if "opengd77_rt3s".contains(radio_model) {
        return opengd77_rt3s::read(opt);
    } else {
        cprintln!(ANSI_C_RED, "Unsupported radio model for operation: read");
        print_supported_radios_read();
        return Err("Bad radio model".into())
    }
}

pub fn write_codeplug(codeplug: &Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut write_functions: HashMap<&str, fn(&Codeplug, &Opt) -> Result<(), Box<dyn Error>>>
        = HashMap::new();
    write_functions.insert("anytone_x78", anytone_x78::write);
    write_functions.insert("opengd77_rt3s", opengd77_rt3s::write);
    //write_functions.insert("chirp_generic", chirp_generic::write);

    if opt.radio.is_none() {
        cprintln!(ANSI_C_RED, "Radio model is required for operation: write");
        return Err("Bad radio model".into());
    }

    // validate the output path
    if opt.output.is_none() {
        cprintln!(ANSI_C_RED, "Output path is required for operation: write");
        return Err("Bad output path".into());
    }

    // get the radio model
    let radio_model = opt.radio.as_ref().unwrap();

    if let Some(write_function) = write_functions.get(radio_model.as_str()) {
        return write_function(codeplug, opt);
    } else {
        cprintln!(ANSI_C_RED, "Unsupported radio model for operation: write");
        cprintln!(ANSI_C_YLW, "Operation write supports the following radio models:");
        for (radio_model, _) in write_functions.iter() {
            cprintln!(ANSI_C_YLW, "    {}", radio_model);
        }
        return Err("Bad radio model".into());
    }
}

// fn print_supported_radios_validate() {
//     eprintln!("Operation validate supports the following radio models:");
//     // eprintln!("    anytone_x78 - Anytone AT-D878UV, etc");
// }

pub fn validate_codeplug(codeplug: &Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // generic validation
    validate_generic(codeplug, opt)?;
    // validate the radio
    // if opt.radio.is_none() {
    //     cprintln!(ANSI_C_RED, "Radio model is required for operation: validate");
    //     return Err("Bad radio model".into());
    // }
    // let radio_model = opt.radio.as_ref().unwrap();

    // // search for the radio model in the supported radios
    // if "anytone_x78".contains(radio_model) {
    //     return anytone_x78::validate(codeplug, opt);
    // } else {
    //     cprintln!(ANSI_C_RED, "Unsupported radio model for operation: validate");
    //     print_supported_radios_read();
    //     return Err("Bad radio model".into())
    // }
    Ok(())
}
