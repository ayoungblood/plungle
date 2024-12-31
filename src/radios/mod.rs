// src/radios/mod.rs

//use std::path::Path;
use std::error::Error;
use crate::Opt;

use crate::structures::Codeplug;

use crate::dprintln;
use crate::cprintln;
use crate::function;

mod anytone_x78;

fn print_supported_radios() {
    eprintln!("Operation read supports the following radio models:");
    eprintln!("    anytone_x78");
}

pub fn read_codeplug(opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // validate the radio
    if opt.radio.is_none() {
        cprintln!(31, "Radio model is required for operation: read");
        print_supported_radios();
        return Err("Bad radio model".into());
    }
    let radio_model = opt.radio.as_ref().unwrap();
    // validate the input path
    if opt.input.is_none() {
        cprintln!(31, "Input path is required for operation: read");
        return Err("Bad input path".into());
    }

    // search for the radio model in the supported radios
    if "anytone_x78".contains(radio_model) {
        return anytone_x78::read(opt);
    } else {
        cprintln!(31, "Unsupported radio model for operation: read");
        print_supported_radios();
        return Err("Bad radio model".into())
    }
}
