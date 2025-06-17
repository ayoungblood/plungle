// src/bandplan.rs

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;

use crate::*;
use crate::frequency::Frequency;

/// Band
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Band {
    pub name: String,
    pub ranges: Vec<(Frequency, Frequency)>,
    pub nominal_offsets: Option<Vec<Frequency>>,
    pub is_amateur: bool,
}

/// Bandplan
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Bandplan {
    pub name: String,
    pub source: Option<String>,
    pub bands: Vec<Band>,
}

// load a bandplan from a file
pub fn load_bandplan(opt: &Opt) -> Result<Bandplan, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    let json_path = PathBuf::from("bandplan.json");
    if json_path.exists() {
        // first look for bandplan.json in the current directory
        let json_str = std::fs::read_to_string(&json_path)?;
        let mut bandplan: Bandplan = serde_json::from_str(&json_str)?;
        bandplan.source = Some(json_path.to_str().unwrap().to_string());
        return Ok(bandplan);
    } else {
        // otherwise, look for default_bandplan.json relative to the binary:
        // src/bandplan/default_bandplan.json (for development)
        // default_bandplan.json (for production)
        let default_json_path = PathBuf::from("src/bandplan/default_bandplan.json");
        if default_json_path.exists() {
            let json_str = std::fs::read_to_string(&default_json_path)?;
            let mut bandplan: Bandplan = serde_json::from_str(&json_str)?;
            bandplan.source = Some(default_json_path.to_str().unwrap().to_string());
            return Ok(bandplan);
        }
        let default_json_path = PathBuf::from("default_bandplan.json");
        if default_json_path.exists() {
            let json_str = std::fs::read_to_string(&default_json_path)?;
            let mut bandplan: Bandplan = serde_json::from_str(&json_str)?;
            bandplan.source = Some(default_json_path.to_str().unwrap().to_string());
            return Ok(bandplan);
        }
    }
    return Err("Bandplan not found".into());
}

pub fn get_band<'a>(bandplan: &'a Bandplan, frequency: &Frequency) -> Option<&'a Band> {
    for band in &bandplan.bands {
        for range in &band.ranges {
            if *frequency >= range.0 && *frequency <= range.1 {
                return Some(band);
            }
        }
    }
    None
}
