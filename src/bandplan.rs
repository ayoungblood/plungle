// src/bandplan.rs

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::*;
use std::io::Write;

/// Band
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Band {
    pub name: String,
    pub ranges: Vec<(Decimal, Decimal)>,
    pub nominal_offsets: Option<Vec<Decimal>>,
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
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    let bandplan_path_default: PathBuf = ["bandplan.toml"].iter().collect();
    // create the bandplan file if it doesn't exist
    try_write_bandplan(&bandplan_path_default, opt)?;
    // load the bandplan
    let toml_str = std::fs::read_to_string(&bandplan_path_default)?;
    let mut bandplan: Bandplan = toml::from_str(&toml_str)?;
    bandplan.source = Some(bandplan_path_default.to_str().unwrap().to_string());
    Ok(bandplan)
}

// write a minimal bandplan to a file if it doesn't exist
pub fn try_write_bandplan(path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    // if the path doesn't exist, create the default bandplan
    if !Path::new(path).exists() {
        uprintln!(opt, Stderr, None, 1, "Default bandplan not found, creating: {}", path.display());
        let bandplan = Bandplan {
            name: String::from("Default Bandplan"),
            source: None,
            bands: vec![
                Band {
                    name: String::from("Amateur 10m"),
                    ranges: vec![(Decimal::from(28_000_000), Decimal::from(29_700_000))],
                    nominal_offsets: Some(vec![Decimal::from(100_000)]), // 100 kHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 6m"),
                    ranges: vec![(Decimal::from(50_000_000), Decimal::from(54_000_000))],
                    nominal_offsets: Some(vec![Decimal::from(500_000), Decimal::from(1_000_000)]), // 500 kHz, 1 MHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 2m"),
                    ranges: vec![(Decimal::from(144_000_000), Decimal::from(148_000_000))],
                    nominal_offsets: Some(vec![Decimal::from(600_000)]), // 600 kHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("MURS"),
                    ranges: vec![(Decimal::from(151_820_000), Decimal::from(151_940_000)), (Decimal::from(154_570_000), Decimal::from(154_600_000))],
                    nominal_offsets: None,
                    is_amateur: false,
                },
                Band {
                    name: String::from("Amateur 1.25m"),
                    ranges: vec![(Decimal::from(219_000_000), Decimal::from(225_000_000))],
                    nominal_offsets: Some(vec![Decimal::from(1_600_000)]), // 1.6 MHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 70cm"),
                    ranges: vec![(Decimal::from(420_000_000), Decimal::from(450_000_000))],
                    nominal_offsets: Some(vec![Decimal::from(5_000_000)]), // 5 MHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("FRS/GMRS"),
                    ranges: vec![(Decimal::from(462_550_000), Decimal::from(462_725_000)), (Decimal::from(467_550_000), Decimal::from(467_725_000))],
                    nominal_offsets: Some(vec![Decimal::from(5_000_000)]), // 5 MHz
                    is_amateur: false,
                },
                Band {
                    name: String::from("Amateur 33cm"),
                    ranges: vec![(Decimal::from(902_000_000), Decimal::from(928_000_000))],
                    nominal_offsets: Some(vec![Decimal::from(12_000_000), Decimal::from(25_000_000)]), // 12 MHz, 25 MHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 23cm"),
                    ranges: vec![(Decimal::from(1_240_000_000), Decimal::from(1_300_000_000))],
                    nominal_offsets: Some(vec![Decimal::from(12_000_000), Decimal::from(20_000_000)]), // 12 MHz, 20 MHz
                    is_amateur: true,
                }
            ],
        };
        // write the bandplan
        let mut f = File::create(path).expect(format!("Failed to create file: {}", path.display()).as_str());
        f.write_all(toml::to_string_pretty(&bandplan).unwrap().as_bytes()).expect("Failed to write bandplan");
    }
    Ok(())
}

pub fn get_band(bandplan: &Bandplan, frequency: Decimal) -> Option<&Band> {
    for band in &bandplan.bands {
        for range in &band.ranges {
            if frequency >= range.0 && frequency <= range.1 {
                return Some(band);
            }
        }
    }
    None
}
