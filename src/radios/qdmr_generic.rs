// src/radios/qdmr_generic.rs

use std::error::Error;
use std::path::PathBuf;
use std::sync::OnceLock;
use saphyr::{Yaml, LoadableYamlNode};
use pretty_yaml::{config::FormatOptions, format_text};

use crate::*;
use crate::structures::*;
use frequency::Frequency;

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

// Extend Yaml functionality to allow an integer or float to be handled as a float
trait NumericYamlExt {
    fn as_any_number(&self) -> Result<f64, Box<dyn Error>>;
}

impl NumericYamlExt for Yaml<'_> {
    fn as_any_number(&self) -> Result<f64, Box<dyn Error>> {
        if let Some(f64) = self.as_floating_point() {
            Ok(f64)
        } else if let Some(i64) = self.as_integer() {
            Ok(i64 as f64)
        } else {
            Err("not a number".into())
        }
    }
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
//     rxFrequency: float or integer MHz
//     txFrequency: float or integer MHz
//     rxOnly: bool
//     admit: [Always,Free,Tone] for analog
//     bandwidth: [Wide,Narrow]
//     power: !<!default> "" or [Max,High,Mid,Low,Min]
//     timeout: !<!default> "" or integers seconds, 0 for off
//     vox: !<!default> "" or 1-10, 0 for off
//     squelch: !<!default> "" or [1-10], 0 for open
//     (optional): rxTone: {ctcss: float Hz} or {dcs: integer} (negative for inverted)
//     (optional): txTone: {ctcss: float Hz} or {dcs: integer} (negative for inverted)
//   - digital:
//     id: string(ch<n>)
//     name: string
//     rxFrequency: float or integer MHz
//     txFrequency: float or integer MHz
//     rxOnly: bool
//     admit: [Always,Free,ColorCode] for digital
//     colorCode: integer [0-15]
//     timeSlot: [TS1,TS2]
//     radioId: !<!default> "" or id<n>
//     (optional) groupList: group id string(grp<n>)
//     (optional) contact: contact id string(cont<n>)
//     power: !<!default> "" or [Max,High,Mid,Low,Min]
//     timeout: !<!default> "" or integers seconds, 0 for off
//     vox: !<!default> "" or 1-10, 0 for off
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

fn parse_channel_record(opt: &Opt, yaml: &Yaml) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", yaml);
    let mut channel = Channel::default();
    let inner: &Yaml;
    if yaml.as_mapping_get("analog").is_some() {
        channel.mode = ChannelMode::FM;
        inner = yaml.as_mapping_get("analog").unwrap();
    } else if yaml.as_mapping_get("digital").is_some() {
        channel.mode = ChannelMode::DMR;
        inner = yaml.as_mapping_get("digital").unwrap();
    } else {
        return Err("Invalid channel record".into());
    }
    //println!("inner: {:?}", inner);
    channel.index = inner.as_mapping_get("id").unwrap().as_str().unwrap().strip_prefix("ch").unwrap().parse::<usize>().unwrap();
    channel.name = inner.as_mapping_get("name").unwrap().as_str().unwrap().to_string();
    channel.frequency_rx = Frequency::from_mhz(inner.as_mapping_get("rxFrequency").unwrap().as_any_number()?);
    channel.frequency_tx = Frequency::from_mhz(inner.as_mapping_get("txFrequency").unwrap().as_any_number()?);
    channel.rx_only = inner.as_mapping_get("rxOnly").unwrap().as_bool().unwrap();
    // some fields in yaml have fancy tags for default, but we just assume that if it's not a value, it's a default
    if let Some(timeout) = inner.as_mapping_get("timeout").unwrap().as_integer() {
        if timeout <= 0 {
            channel.tx_tot = Timeout::Infinite;
        } else {
            channel.tx_tot = Timeout::Seconds(timeout.try_into().unwrap());
        }
    }
    if let Some(power) = inner.as_mapping_get("power").unwrap().as_str() {
        channel.power = match power {
            "Max"  => Power::Watts(7.0),
            "High" => Power::Watts(5.0),
            "Mid"  => Power::Watts(2.5),
            "Low"  => Power::Watts(1.0),
            "Min"  => Power::Watts(0.5),
            _ => return Err("Invalid power level".into()),
        };
    }
    // @TODO ADMIT
    // mode-specific fields
    if channel.mode == ChannelMode::FM {
        channel.fm = Some(FmChannel {
            bandwidth: match inner.as_mapping_get("bandwidth").unwrap().as_str().unwrap() {
                "Wide" => Frequency::from_khz(25.0),
                "Narrow" => Frequency::from_khz(12.5),
                _ => return Err("Invalid bandwidth".into()),
            },
            squelch: match inner.as_mapping_get("squelch").unwrap().as_integer() {
                Some(sq) => Squelch::Percent(sq as u8 * 10),
                None => Squelch::default(),
            },
            tone_rx: None,
            tone_tx: None,
        });
        println!("{:?}\n", inner.as_mapping_get("squelch").unwrap().as_integer());
    } else if channel.mode == ChannelMode::DMR {
        channel.dmr = Some(DmrChannel {
            timeslot: 0,
            color_code: 0,
            talkgroup: None,
            talkgroup_list: None,
            id_name: None,
        });
    }

    Ok(channel)
}

pub fn read(opt: &Opt, input_path: &PathBuf) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug::default();

    // check that the input path is a file
    if !input_path.is_file() {
        uprintln!(opt, Stderr, Color::Red, None, "You lied to me when you told me this was a file: {}", input_path.display());
        return Err("Bad input path".into());
    }

    uprintln!(opt, Stderr, None, 3, "Reading {}", input_path.display());
    let yaml_str = std::fs::read_to_string(input_path)?;
    let yaml = &(Yaml::load_from_str(&yaml_str).unwrap())[0];

    let options = FormatOptions::default();
    let formatted_yaml = format_text(&yaml_str, &options)?;
    uprintln!(opt, Stderr, None, 4, "formatted_yaml:\n{}", formatted_yaml);

    for channel in yaml.as_mapping_get("channels").unwrap().as_vec().unwrap() {
        let channel = parse_channel_record(opt, channel)?;
        codeplug.channels.push(channel);
    }

    codeplug.source = format!("qdmr_v{}", yaml["version"].as_str().unwrap_or("ERR"));

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////
