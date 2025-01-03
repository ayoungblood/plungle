// src/radios/mod.rs

use std::error::Error;
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

fn print_supported_radios_write() {
    eprintln!("Operation write supports the following radio models:");
    eprintln!("    opengd77_rt3s - Retevis RT3S with OpenGD77 firmware");
    //eprintln!("    chirp_generic - Generic CHIRP CSV")
}

pub fn write_codeplug(codeplug: &Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    // validate the radio
    if opt.radio.is_none() {
        cprintln!(ANSI_C_RED, "Radio model is required for operation: write");
        return Err("Bad radio model".into());
    }
    let radio_model = opt.radio.as_ref().unwrap();
    // validate the output path
    if opt.output.is_none() {
        cprintln!(ANSI_C_RED, "Output path is required for operation: write");
        return Err("Bad output path".into());
    }

    // search for the radio model in the supported radios
    if "opengd77_rt3s".contains(radio_model) {
        return opengd77_rt3s::write(codeplug, opt);
    // } else if "chirp_generic".contains(radio_model) {
    //     return chirp_generic::write(codeplug, opt);
    } else {
        cprintln!(ANSI_C_RED, "Unsupported radio model for operation: write");
        print_supported_radios_write();
        return Err("Bad radio model".into())
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
