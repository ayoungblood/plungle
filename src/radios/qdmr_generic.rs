// src/radios/qdmr_generic.rs

use std::error::Error;
use std::path::PathBuf;
use std::sync::OnceLock;
use saphyr::{Yaml, LoadableYamlNode, YamlEmitter, Scalar, Sequence, Mapping};
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
// The format is very well documented here: https://dm3mat.darc.de/qdmr/manual/ch03.html
// As of QDMR 0.11.2, the format is as follows:
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

fn parse_configuration(opt: &Opt, yaml: &Yaml) -> () {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, THEME.noise, 5, "{:?}", yaml);

    println!("{:?}", yaml.as_mapping_get("radioIDs").unwrap());
    // for dmr in  {
    //     println!("{:?}", dmr);
    // }
    ()
}

fn parse_talkgroup_record(opt: &Opt, yaml: &Yaml) -> Result<DmrTalkgroup, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, THEME.noise, 5, "{:?}", yaml);

    Ok(DmrTalkgroup {
        index: yaml.as_mapping_get("dmr").unwrap()
            .as_mapping_get("id").unwrap()
            .as_str().unwrap()
            .strip_prefix("cont").unwrap()
            .parse::<usize>().unwrap(),
        id: yaml.as_mapping_get("dmr").unwrap()
            .as_mapping_get("number").unwrap()
            .as_integer().unwrap() as u32,
        name: yaml.as_mapping_get("dmr").unwrap()
            .as_mapping_get("name").unwrap()
            .as_str().unwrap()
            .to_string(),
        call_type: match yaml.as_mapping_get("dmr").unwrap()
            .as_mapping_get("type").unwrap()
            .as_str().unwrap() {
                "GroupCall" => DmrTalkgroupCallType::Group,
                "PrivateCall" => DmrTalkgroupCallType::Private,
                "AllCall" => DmrTalkgroupCallType::AllCall,
                _ => return Err("Unknown call type".into()),
            },
        alert: yaml.as_mapping_get("dmr").unwrap()
            .as_mapping_get("ring").unwrap()
            .as_bool().unwrap(),
    })
}

fn parse_talkgroup_list_record(opt: &Opt, yaml: &Yaml, codeplug: &Codeplug) -> Result<DmrTalkgroupList, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, THEME.noise, 5, "{:?}", yaml);

    let mut talkgroup_list = DmrTalkgroupList {
        index: yaml.as_mapping_get("id").unwrap()
            .as_str().unwrap()
            .strip_prefix("grp").unwrap()
            .parse::<usize>().unwrap(),
        name: yaml.as_mapping_get("name").unwrap()
            .as_str().unwrap()
            .to_string(),
        talkgroups: Vec::new(),
    };
    // loop through contact IDs, translate to
    for id in yaml.as_mapping_get("contacts").unwrap().as_vec().unwrap() {
        let index: usize = id.as_str().unwrap()
            .strip_prefix("cont").unwrap()
            .parse::<usize>().unwrap();
        // find talkgroup by index
        let talkgroup = codeplug.talkgroups.iter().find(|tg| tg.index == index);
        if let Some(tg) = talkgroup {
            talkgroup_list.talkgroups.push(tg.clone());
        } else {
            uprintln!(opt, Stderr, THEME.warn, None, "Talkgroup not found: {}", index);
        }
    }
    Ok(talkgroup_list)
}

fn parse_tone(yaml: Option<&Yaml>) -> Option<Tone> {
    if let Some(yaml) = yaml {
        if let Some(yaml_ctcss) = yaml.as_mapping_get("ctcss") {
            // qdmr floats aren't very precise, round to 1 decimal place
            let tone = (yaml_ctcss.as_any_number().unwrap() * 10.0).round() / 10.0;
            Some(Tone::Ctcss(tone))
        } else if let Some(yaml_dcs) = yaml.as_mapping_get("dcs") {
            // qdmr uses negative integers for inverted DCS tones
            let dcs_int = yaml_dcs.as_integer().unwrap();
            Some(Tone::Dcs(format!("D{:03}{}", dcs_int.abs(), if dcs_int < 0 { "I" } else { "N" })))
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_channel_record(opt: &Opt, yaml: &Yaml, all_yaml: &Yaml) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, THEME.noise, 5, "{:?}", yaml);

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
    // parse common fields
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
    if let Some(scan_list_ref) = inner.as_mapping_get("scanListRef") {
        let scan_list_id = scan_list_ref.as_str().unwrap();
        // scanlists haven't been parsed yet, so we have to search scanlists by ID
        for scanlist_yaml in all_yaml.as_mapping_get("scanLists").unwrap().as_vec().unwrap() {
            if scanlist_yaml.as_mapping_get("id").unwrap().as_str().unwrap() == scan_list_id {
                channel.scan = Some(Scan::ScanList(scanlist_yaml.as_mapping_get("name").unwrap().as_str().unwrap().to_string()));
                break;
            }
        }
    }
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
            tone_rx: parse_tone(inner.as_mapping_get("rxTone")),
            tone_tx: parse_tone(inner.as_mapping_get("txTone")),
        });
        channel.tx_permit = match inner.as_mapping_get("admit").unwrap().as_str().unwrap() {
            "Always" => Some(TxPermit::Always),
            "Free" => Some(TxPermit::ChannelFree),
            "Tone" => Some(TxPermit::CtcssDcsDifferent), // @TODO: verify this
            _ => return Err("Invalid TX permit".into()),
        };
    } else if channel.mode == ChannelMode::DMR {
        channel.dmr = Some(DmrChannel {
            timeslot: inner.as_mapping_get("timeSlot").unwrap()
                .as_str().unwrap().strip_prefix("TS").unwrap()
                .parse::<u8>().unwrap(),
            color_code: inner.as_mapping_get("colorCode").unwrap()
                .as_integer().unwrap() as u8,
            talkgroup: None,
            talkgroup_list: None,
            id_name: None,
        });
        channel.tx_permit = match inner.as_mapping_get("admit").unwrap().as_str().unwrap() {
            "Always" => Some(TxPermit::Always),
            "Free" => Some(TxPermit::ChannelFree),
            "ColorCode" => Some(TxPermit::ColorCodeSame), // @TODO: verify this
            _ => return Err("Invalid TX permit".into()),
        };
    }

    Ok(channel)
}

fn parse_scanlist_record(opt: &Opt, yaml: &Yaml, codeplug: &Codeplug) -> Result<ScanList, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, THEME.noise, 5, "{:?}", yaml);

    let mut scanlist = ScanList {
        index: yaml.as_mapping_get("id").unwrap()
            .as_str().unwrap()
            .strip_prefix("scan").unwrap()
            .parse::<usize>().unwrap(),
        name: yaml.as_mapping_get("name").unwrap()
            .as_str().unwrap()
            .to_string(),
        channels: Vec::new(),
    };
    // loop through channel IDs and get channel names
    for id in yaml.as_mapping_get("channels").unwrap().as_vec().unwrap() {
        let index: usize = id.as_str().unwrap()
            .strip_prefix("ch").unwrap()
            .parse::<usize>().unwrap();
        // find channel by index
        let channel = codeplug.channels.iter().find(|c| c.index == index);
        if let Some(channel) = channel {
            scanlist.channels.push(channel.name.clone());
        } else {
            uprintln!(opt, Stderr, THEME.warn, None, "Channel not found: {}", index);
        }
    }

    Ok(scanlist)
}

fn parse_zone_record(opt: &Opt, yaml: &Yaml, codeplug: &Codeplug) -> Result<Zone, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, THEME.noise, 5, "{:?}", yaml);

    let mut zone = Zone {
        index: yaml.as_mapping_get("id").unwrap()
            .as_str().unwrap()
            .strip_prefix("zone").unwrap()
            .parse::<usize>().unwrap(),
        name: yaml.as_mapping_get("name").unwrap()
            .as_str().unwrap()
            .to_string(),
        channels: Vec::new(),
    };

    // qdmr does something weird with zones and has them split into A and B groups
    // parse both and stuff into a single zone
    for id in yaml.as_mapping_get("A").unwrap().as_vec().unwrap() {
        let index: usize = id.as_str().unwrap()
            .strip_prefix("ch").unwrap()
            .parse::<usize>().unwrap();
        let channel = codeplug.channels.iter().find(|c| c.index == index);
        if let Some(channel) = channel {
            zone.channels.push(channel.name.clone());
        } else {
            uprintln!(opt, Stderr, THEME.warn, None, "Channel not found: {}", id.as_str().unwrap());
        }
    }
    for id in yaml.as_mapping_get("B").unwrap().as_vec().unwrap() {
        let index: usize = id.as_str().unwrap()
            .strip_prefix("ch").unwrap()
            .parse::<usize>().unwrap();
        let channel = codeplug.channels.iter().find(|c| c.index == index);
        if let Some(channel) = channel {
            zone.channels.push(channel.name.clone());
        } else {
            uprintln!(opt, Stderr, THEME.warn, None, "Channel not found: {}", id.as_str().unwrap());
        }
    }

    Ok(zone)
}

pub fn read(opt: &Opt, input_path: &PathBuf) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
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
    uprintln!(opt, Stderr, THEME.noise, 5, "{}", formatted_yaml);

    parse_configuration(opt, &yaml);

    if let Some(talkgroups_vec) = yaml.as_mapping_get("contacts")
        .and_then(|yaml_talkgroups| yaml_talkgroups.as_vec()) {
        for yaml_talkgroup in talkgroups_vec {
            let talkgroup = parse_talkgroup_record(opt, yaml_talkgroup)?;
            codeplug.talkgroups.push(talkgroup);
        }
    }

    if let Some(talkgroup_lists_vec) = yaml.as_mapping_get("groupLists")
        .and_then(|yaml_talkgroup_lists| yaml_talkgroup_lists.as_vec()) {
        for yaml_talkgroup_list in talkgroup_lists_vec {
            let talkgroup_list = parse_talkgroup_list_record(opt, yaml_talkgroup_list, &codeplug)?;
            codeplug.talkgroup_lists.push(talkgroup_list);
        }
    }

    if let Some(channels_vec) = yaml.as_mapping_get("channels")
        .and_then(|yaml_channels| yaml_channels.as_vec()) {
        for yaml_channel in channels_vec {
            let channel = parse_channel_record(opt, yaml_channel, yaml)?;
            codeplug.channels.push(channel);
        }
    } else {
        uprintln!(opt, Stderr, THEME.err, None, "Channels vector not found in codeplug");
        return Err("Invalid codeplug".into());
    }

    if let Some(scanlists_vec) = yaml.as_mapping_get("scanLists")
        .and_then(|yaml_scanlists| yaml_scanlists.as_vec()) {
        for yaml_scanlist in scanlists_vec {
            let scanlist = parse_scanlist_record(opt, yaml_scanlist, &codeplug)?;
            codeplug.scanlists.push(scanlist);
        }
    }

    if let Some(zones_vec) = yaml.as_mapping_get("zones")
        .and_then(|yaml_zones| yaml_zones.as_vec()) {
        for yaml_zone in zones_vec {
            let zone = parse_zone_record(opt, yaml_zone, &codeplug)?;
            codeplug.zones.push(zone);
        }
    }

    codeplug.source = format!("qdmr_v{}", yaml["version"].as_str().unwrap_or("ERR"));

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

fn write_settings(opt: &Opt) -> Result<Mapping, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut settings_map = Mapping::new();
    settings_map.insert(
        Yaml::Value(Scalar::String("introLine1".into())),
        Yaml::Value(Scalar::String("".into())),
    );
    settings_map.insert(
        Yaml::Value(Scalar::String("introLine2".into())),
        Yaml::Value(Scalar::String("".into())),
    );
    settings_map.insert(
        Yaml::Value(Scalar::String("micLevel".into())),
        Yaml::Value(Scalar::Integer(3)),
    );
    settings_map.insert(
        Yaml::Value(Scalar::String("speech".into())),
        Yaml::Value(Scalar::Boolean(false)),
    );
    settings_map.insert(
        Yaml::Value(Scalar::String("power".into())),
        Yaml::Value(Scalar::String("High".into())),
    );
    settings_map.insert(
        Yaml::Value(Scalar::String("squelch".into())),
        Yaml::Value(Scalar::Integer(1)),
    );
    settings_map.insert(
        Yaml::Value(Scalar::String("vox".into())),
        Yaml::Value(Scalar::Integer(0)),
    );
    settings_map.insert(
        Yaml::Value(Scalar::String("tot".into())),
        Yaml::Value(Scalar::Integer(0)),
    );

    Ok(settings_map)
}

fn write_radio_ids<'a>(opt: &Opt, codeplug: &'a Codeplug) -> Result<Sequence<'a>, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let radio_ids_vec = Sequence::new();
    if let Some(config) = &codeplug.config {
        if let Some(dmr_config) = &config.dmr_configuration {
            for dmr_id in &dmr_config.id_list {
                println!("DMR ID: {:?}", dmr_id);
            }
        }
    }

    Ok(radio_ids_vec)
}

fn write_talkgroups(opt: &Opt) -> Result<Sequence, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let contacts_vec = Sequence::new();

    Ok(contacts_vec)
}

fn write_talkgroup_lists(opt: &Opt) -> Result<Sequence, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let talkgroup_lists_vec = Sequence::new();

    Ok(talkgroup_lists_vec)
}

fn write_channels(opt: &Opt) -> Result<Sequence, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let channels_vec = Sequence::new();

    Ok(channels_vec)
}

fn write_zones(opt: &Opt) -> Result<Sequence, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let zones_vec = Sequence::new();

    Ok(zones_vec)
}

pub fn write(opt: &Opt, codeplug: &Codeplug, output_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}:{}()", file!(), line!(),function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    // if the output path exists, complain
    if output_path.exists() {
        uprintln!(opt, Stderr, THEME.err, None, "Output path already exists: {}", output_path.display());
        return Err("Output path already exists".into());
    }

    // create the top level map
    let mut yaml_map = Mapping::new();
    yaml_map.insert(
        Yaml::Value(Scalar::String("version".into())),
        Yaml::Value(Scalar::String("0.11.2".into())),
    );

    // add the settings map
    yaml_map.insert(
        Yaml::Value(Scalar::String("settings".into())),
        Yaml::Mapping(write_settings(opt)?),
    );

    // add the radio IDs vec
    yaml_map.insert(
        Yaml::Value(Scalar::String("radioIDs".into())),
        Yaml::Sequence(write_radio_ids(opt, codeplug)?),
    );

    // add the talkgroups vec
    yaml_map.insert(
        Yaml::Value(Scalar::String("contacts".into())),
        Yaml::Sequence(write_talkgroups(opt)?),
    );

    // add the talkgroup lists vec
    yaml_map.insert(
        Yaml::Value(Scalar::String("groupLists".into())),
        Yaml::Sequence(write_talkgroup_lists(opt)?),
    );

    // add the channels vec
    yaml_map.insert(
        Yaml::Value(Scalar::String("channels".into())),
        Yaml::Sequence(write_channels(opt)?),
    );

    // add the zones vec
    yaml_map.insert(
        Yaml::Value(Scalar::String("zones".into())),
        Yaml::Sequence(write_zones(opt)?),
    );

    // add the commercial map for parity
    let mut commercial_map: Mapping = Mapping::new();
    commercial_map.insert(
        Yaml::Value(Scalar::String("encryptionKeys".into())),
        Yaml::Sequence(Sequence::new()),
    );
    yaml_map.insert(
        Yaml::Value(Scalar::String("commercial".into())),
        Yaml::Mapping(commercial_map),
    );

    // wrap the map in a Yaml::Mapping and serialize
    let yaml = Yaml::Mapping(yaml_map);
    let mut yaml_str = String::new();
    let mut emitter = YamlEmitter::new(&mut yaml_str);
    emitter.compact(false);
    emitter.dump(&yaml)?;

    // @TODO remove me
    println!("{}", yaml_str);

    // Write the YAML string to the output file
    std::fs::write(output_path, yaml_str)?;

    Ok(())
}
