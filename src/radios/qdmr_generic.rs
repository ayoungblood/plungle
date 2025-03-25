// src/radios/qdmr_generic.rs

use std::error::Error;
// use std::fs;
use std::path::PathBuf;
// use std::path::Path;
// use std::collections::HashMap;
// use rust_decimal::prelude::*;
use std::sync::OnceLock;
// use std::cmp::{max, min};
use saphyr::{Yaml};

use crate::*;
use crate::structures::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::FM, structures::ChannelMode::DMR];
        props.channels_max = 4000;
        props.channel_name_width_max = 16;
        props.zones_max = 4000;
        props.zone_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// QDMR uses YAML for storing codeplugs
// As of QDMR 0.11.2, the format is as follows
// version: version string (0.11.2)
// settings:
//   introLine1: string
//   introLine2: string
//   micLevel: integer [1,10]
//   speech: bool
//   squelch: integer [0,10]
//   vox: integer [0,10] (0 is disabled)
//   tot: default TOT
//   defaultID: default DMR ID
// radioIDs: array of DMR IDs
//   dmr: {id: string(id<n>), name: string, number: integer}
// contacts:
//   dmr: {id: string(cont<n>), name: string, ring: bool, type: [GroupCall,PrivateCall,AllCall], number: integer}
// groupLists:
//   dmr: {id: string(grp<n>), name: string, contacts: [array of contact ids]}
// channels:
//   - analog:
//     id: string(ch<n>)
//     name: string
//     rxFrequency: float MHz
//     txFrequency: float MHz
//     rxOnly: bool
//     admit: [Always,Free,Tone] for analog
//     bandwidth: [Wide,Narrow]
//     power: !<!default> "" or [Max,Mid,Low,...]
//     timeout: !<!default> "" or integers seconds, 0 for off
//     vox: !<!default> "" or ??
//     squelch: !<!default> "" or [0-10]
//     (optional): rxTone: {ctcss: float Hz} or {dcs: integer} (negative for inverted)
//     (optional): txTone: {ctcss: float Hz} or {dcs: integer} (negative for inverted)
//   - digital:
//     id: string(ch<n>)
//     name: string
//     rxFrequency: float MHz
//     txFrequency: float MHz
//     rxOnly: bool
//     admit: [Always,Free,ColorCode] for digital
//     colorCode: integer
//     timeSlot: [TS1,TS2]
//     radioId: !<!default> "" or ??
//     (optional) groupList: group id string(grp<n>)
//     (optional) contact: contact id string(cont<n>)
//     power: !<!default> "" or [Max,Mid,Low,...]
//     timeout: !<!default> "" or integers seconds, 0 for off
//     vox: !<!default> "" or ??
// zones:
//   []
// scanLists:
//   - id: string(scan<n>)
//     name: string
//     channels: [array of channel ids]
// commercial:
//   encryptionKeys:
//     []
// ...

// READ ///////////////////////////////////////////////////////////////////////

pub fn read(input_path: &PathBuf, opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug::default();

    // check that the input path is a file
    if !input_path.is_file() {
        uprintln!(opt, Stderr, Color::Red, None, "You lied to me when you told me this was a file: {}", input_path.display());
        return Err("Bad input path".into());
    }

    uprintln!(opt, Stderr, None, 3, "Reading {}", input_path.display());
    let yaml_str = std::fs::read_to_string(input_path)?;
    let yaml = &(Yaml::load_from_str(&yaml_str)?)[0];

    codeplug.source = format!("qdmr_v{}", yaml["version"].as_str().unwrap_or("ERR"));

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////


