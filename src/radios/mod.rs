// src/radios/mod.rs

use std::error::Error;
use std::collections::HashMap;

use crate::Opt;
use crate::structures::Codeplug;
use crate::*;
use crate::validate::validate_generic;

mod anytone_x78;
mod opengd77_rt3s;
mod chirp_generic;
mod ailunce_hd1;

pub fn read_codeplug(opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut read_functions: HashMap<&str, fn(&Opt) -> Result<Codeplug, Box<dyn Error>>>
        = HashMap::new();
    read_functions.insert("anytone_x78", anytone_x78::read);
    read_functions.insert("opengd77_rt3s", opengd77_rt3s::read);
    read_functions.insert("chirp_generic", chirp_generic::read);
    read_functions.insert("ailunce_hd1", ailunce_hd1::read);

    if opt.radio.is_none() {
        cprintln!(ANSI_C_RED, "Radio model is required for operation: read");
        return Err("Bad radio model".into());
    }

    // validate the input path
    if opt.input.is_none() {
        cprintln!(ANSI_C_RED, "Input path is required for operation: read");
        return Err("Bad input path".into());
    }

    let radio_model = opt.radio.as_ref().unwrap();
    // look up the radio model in the hashmap
    if let Some(read_function) = read_functions.get(radio_model.as_str()) {
        return read_function(opt);
    } else {
        cprintln!(ANSI_C_RED, "Unsupported radio model for operation: read");
        cprintln!(ANSI_C_YLW, "Operation \"read\" supports the following radio models:");
        for (radio_model, _) in read_functions.iter() {
            cprintln!(ANSI_C_YLW, "    {}", radio_model);
        }
        return Err("Bad radio model".into());
    }
}

pub fn write_codeplug(codeplug: &Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut write_functions: HashMap<&str, fn(&Codeplug, &Opt) -> Result<(), Box<dyn Error>>>
        = HashMap::new();
    write_functions.insert("anytone_x78", anytone_x78::write);
    write_functions.insert("opengd77_rt3s", opengd77_rt3s::write);
    write_functions.insert("chirp_generic", chirp_generic::write);
    write_functions.insert("ailunce_hd1", ailunce_hd1::write);

    if opt.radio.is_none() {
        cprintln!(ANSI_C_RED, "Radio model is required for operation: write");
        return Err("Bad radio model".into());
    }

    // validate the output path
    if opt.output.is_none() {
        cprintln!(ANSI_C_RED, "Output path is required for operation: write");
        return Err("Bad output path".into());
    }

    let radio_model = opt.radio.as_ref().unwrap();
    // look up the radio model in the hashmap
    if let Some(write_function) = write_functions.get(radio_model.as_str()) {
        return write_function(codeplug, opt);
    } else {
        cprintln!(ANSI_C_RED, "Unsupported radio model for operation: write");
        cprintln!(ANSI_C_YLW, "Operation \"write\" supports the following radio models:");
        for (radio_model, _) in write_functions.iter() {
            cprintln!(ANSI_C_YLW, "    {}", radio_model);
        }
        return Err("Bad radio model".into());
    }
}

pub fn validate_codeplug(codeplug: &Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // generic validation
    validate_generic(codeplug, opt)?;
    Ok(())
}
