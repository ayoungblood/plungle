// src/radios/mod.rs

mod ailunce_hd1;
mod alinco_djmd5t;
mod anytone_x78;
mod chirp_generic;
mod opengd77_rt3s;
mod tyt_mduv390;
mod yaesu_ft3d;

use std::error::Error;
use std::collections::HashMap;

use crate::*;
use crate::structures::Codeplug;

pub fn parse_codeplug(opt: &Opt, model: &String, input: &PathBuf) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut read_functions: HashMap<&str, fn(&Opt, &PathBuf) -> Result<Codeplug, Box<dyn Error>>>
        = HashMap::new();
    read_functions.insert("ailunce_hd1", ailunce_hd1::read);
    read_functions.insert("alinco_djmd5t", alinco_djmd5t::read);
    read_functions.insert("anytone_x78", anytone_x78::read);
    read_functions.insert("chirp_generic", chirp_generic::read);
    read_functions.insert("opengd77_rt3s", opengd77_rt3s::read);
    read_functions.insert("tyt_mduv390", tyt_mduv390::read);
    read_functions.insert("yaesu_ft3d", yaesu_ft3d::read);

    // look up the radio model in the hashmap
    if let Some(read_function) = read_functions.get(model.as_str()) {
        return read_function(opt, input);
    } else {
        uprintln!(opt, Stderr, THEME.err, None, "Unsupported radio model for operation parse: {}", model);
        uprintln!(opt, Stderr, None, None, "Operation \"parse\" supports the following radio models:");
        for (kk, _) in read_functions.iter() {
            uprintln!(opt, Stderr, None, None, "    {}", kk);
        }
        return Err("Bad radio model".into());
    }
}

pub fn generate_codeplug(opt: &Opt, codeplug: &Codeplug, model: &String, output: &PathBuf) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut write_functions: HashMap<&str, fn(&Opt, &Codeplug, &PathBuf) -> Result<(), Box<dyn Error>>>
        = HashMap::new();
    write_functions.insert("ailunce_hd1", ailunce_hd1::write);
    write_functions.insert("alinco_djmd5t", alinco_djmd5t::write);
    write_functions.insert("anytone_x78", anytone_x78::write);
    write_functions.insert("chirp_generic", chirp_generic::write);
    write_functions.insert("opengd77_rt3s", opengd77_rt3s::write);
    write_functions.insert("tyt_mduv390", tyt_mduv390::write);
    write_functions.insert("yaesu_ft3d", yaesu_ft3d::write);

    // look up the radio model in the hashmap
    if let Some(write_function) = write_functions.get(model.as_str()) {
        return write_function(opt, codeplug, output);
    } else {
        uprintln!(opt, Stderr, THEME.err, None, "Unsupported radio model for operation write: {}", model);
        uprintln!(opt, Stderr, None, None, "Operation \"write\" supports the following radio models:");
        for (kk, _) in write_functions.iter() {
            uprintln!(opt, Stderr, None, None, "    {}", kk);
        }
        return Err("Bad radio model".into());
    }
}

pub fn get_properties(opt: &Opt, model: &String) -> Result<structures::RadioProperties, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    // build up a hashmap of function pointers
    let mut properties_functions: HashMap<&str, fn() -> &'static structures::RadioProperties>
        = HashMap::new();
    properties_functions.insert("ailunce_hd1", ailunce_hd1::get_props);
    properties_functions.insert("alinco_djmd5t", alinco_djmd5t::get_props);
    properties_functions.insert("anytone_x78", anytone_x78::get_props);
    properties_functions.insert("chirp_generic", chirp_generic::get_props);
    properties_functions.insert("opengd77_rt3s", opengd77_rt3s::get_props);
    properties_functions.insert("tyt_mduv390", tyt_mduv390::get_props);
    properties_functions.insert("yaesu_ft3d", yaesu_ft3d::get_props);

    // look up the radio model in the hashmap
    if let Some(properties_function) = properties_functions.get(model.as_str()) {
        return Ok(properties_function().clone());
    } else {
        uprintln!(opt, Stderr, THEME.err, None, "Unsupported radio model for operation properties: {}", model);
        uprintln!(opt, Stderr, None, None, "Operation \"properties\" supports the following radio models:");
        for (kk, _) in properties_functions.iter() {
            uprintln!(opt, Stderr, None, None, "    {}", kk);
        }
        return Err("Bad radio model".into());
    }
}
