// src/bandplan.rs

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::*;
use std::io::Write;
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
                    ranges: vec![(Frequency::from_mhz(28.000), Frequency::from_mhz(29.700))],
                    nominal_offsets: Some(vec![Frequency::from_khz(100.0)]), // 100 kHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 6m"),
                    ranges: vec![(Frequency::from_mhz(50.0), Frequency::from_mhz(54.0))],
                    nominal_offsets: Some(vec![Frequency::from_khz(500.0), Frequency::from_mhz(1.0)]), // 500 kHz, 1 MHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 2m"),
                    ranges: vec![(Frequency::from_mhz(144.0), Frequency::from_mhz(148.0))],
                    nominal_offsets: Some(vec![Frequency::from_khz(600.0)]), // 600 kHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("MURS"),
                    ranges: vec![(Frequency::from_mhz(151.82), Frequency::from_mhz(151.94)), (Frequency::from_mhz(154.57), Frequency::from_mhz(154.60))],
                    nominal_offsets: None,
                    is_amateur: false,
                },
                Band {
                    name: String::from("Amateur 1.25m"),
                    ranges: vec![(Frequency::from_mhz(219.0), Frequency::from_mhz(225.0))],
                    nominal_offsets: Some(vec![Frequency::from_mhz(1.6)]), // 1.6 MHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 70cm"),
                    ranges: vec![(Frequency::from_mhz(420.0), Frequency::from_mhz(450.0))],
                    nominal_offsets: Some(vec![Frequency::from_mhz(5.0)]), // 5 MHz
                    is_amateur: true,
                },
                Band {
                    name: String::from("FRS/GMRS"),
                    ranges: vec![(Frequency::from_mhz(462.55), Frequency::from_mhz(462.725)), (Frequency::from_mhz(467.55), Frequency::from_mhz(467.725))],
                    nominal_offsets: Some(vec![Frequency::from_mhz(5.0)]), // 5 MHz
                    is_amateur: false,
                },
                Band {
                    name: String::from("Amateur 33cm"),
                    ranges: vec![(Frequency::from_mhz(902.0), Frequency::from_mhz(928.0))],
                    nominal_offsets: Some(vec![Frequency::from_mhz(12.0), Frequency::from_mhz(25.0)]),
                    is_amateur: true,
                },
                Band {
                    name: String::from("Amateur 23cm"),
                    ranges: vec![(Frequency::from_ghz(1.240), Frequency::from_ghz(1.300))],
                    nominal_offsets: Some(vec![Frequency::from_mhz(12.0), Frequency::from_mhz(20.0)]), // 12 MHz, 20 MHz
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
