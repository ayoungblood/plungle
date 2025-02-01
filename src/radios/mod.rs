// src/radios/mod.rs

use std::error::Error;
use std::collections::HashMap;

use crate::Opt;
use crate::structures::Codeplug;
use crate::*;
use crate::validate::validate_generic;
use crate::validate::validate_specific;
use crate::validate::Complaint;
use crate::validate::print_complaints;

mod anytone_x78;
mod opengd77_rt3s;
mod chirp_generic;
mod ailunce_hd1;

pub fn parse_codeplug(model: &String, input: &PathBuf, opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut read_functions: HashMap<&str, fn(&PathBuf, &Opt) -> Result<Codeplug, Box<dyn Error>>>
        = HashMap::new();
    read_functions.insert("anytone_x78", anytone_x78::read);
    read_functions.insert("opengd77_rt3s", opengd77_rt3s::read);
    read_functions.insert("chirp_generic", chirp_generic::read);
    read_functions.insert("ailunce_hd1", ailunce_hd1::read);

    // look up the radio model in the hashmap
    if let Some(read_function) = read_functions.get(model.as_str()) {
        return read_function(input, opt);
    } else {
        uprintln!(opt, Stderr, Color::Red, None, "Unsupported radio model for operation: read");
        uprintln!(opt, Stderr, None, None, "Operation \"read\" supports the following radio models:");
        for (kk, _) in read_functions.iter() {
            uprintln!(opt, Stderr, None, None, "    {}", kk);
        }
        return Err("Bad radio model".into());
    }
}

pub fn generate_codeplug(codeplug: &Codeplug, model: &String, output: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut write_functions: HashMap<&str, fn(&Codeplug, &PathBuf, &Opt) -> Result<(), Box<dyn Error>>>
        = HashMap::new();
    write_functions.insert("anytone_x78", anytone_x78::write);
    write_functions.insert("opengd77_rt3s", opengd77_rt3s::write);
    write_functions.insert("chirp_generic", chirp_generic::write);
    write_functions.insert("ailunce_hd1", ailunce_hd1::write);

    // look up the radio model in the hashmap
    if let Some(write_function) = write_functions.get(model.as_str()) {
        return write_function(codeplug, output, opt);
    } else {
        uprintln!(opt, Stderr, Color::Red, None, "Unsupported radio model for operation: write");
        uprintln!(opt, Stderr, None, None, "Operation \"write\" supports the following radio models:");
        for (kk, _) in write_functions.iter() {
            uprintln!(opt, Stderr, None, None, "    {}", kk);
        }
        return Err("Bad radio model".into());
    }
}

pub fn validate_codeplug(codeplug: &Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    let mut complaints: Vec<Complaint> = Vec::new();
    // load a band plan
    let bandplan = bandplan::load_bandplan(opt)?;
    // generic validation
    complaints.extend(validate_generic(codeplug, &bandplan, opt).unwrap());
    // radio-specific validation
    let properties = structures::RadioProperties {
        channels_max: 1024,
        channel_name_width_max: 16,
        channel_index_width: 4,
    };
    // specific validation
    complaints.extend(validate_specific(codeplug, &properties, opt).unwrap());
    // combine the complaints
    print_complaints(&complaints, opt);
    Ok(())
}
