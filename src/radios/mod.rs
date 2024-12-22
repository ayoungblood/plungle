// src/radios/mod.rs

use std::path::Path;
use std::error::Error;

use crate::structures::Codeplug;

mod anytone_x78;

pub fn parse(radio_model: &str, input: &Path) -> Result<Codeplug, Box<dyn Error>> {
    if "anytone_x78".contains(radio_model) {
        anytone_x78::parse(input)
    } else {
        Err("Unsupported radio model".into())
    }
}
