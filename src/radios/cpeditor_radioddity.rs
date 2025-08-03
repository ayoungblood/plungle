// src/radios/cpeditor_radioddity.rs

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use serde_json::{json, Value};
use std::fs::File;
use std::path::Path;
use std::io::Read;

use crate::*;
use crate::structures::*;
use crate::frequency::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::FM, structures::ChannelMode::DMR];
        props.channels_max = 4000;
        props.channel_name_width_max = 14;
        props.zones_max = 4000;
        props.zone_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// READ ///////////////////////////////////////////////////////////////////////

fn parse_config(opt: &Opt, json: &Value) -> Option<Configuration> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    let config = Configuration {
        dmr_configuration: Some(DmrConfiguration {
            id_list: vec![
                DmrId {
                    id: json["Basic parameters"]["DMR ID"].as_u64().unwrap_or(0) as u32,
                    name: json["Basic parameters"]["Radio name"].as_str().unwrap_or("").to_string(),
                }
            ]
        }),
    };
    Some(config)
}

fn parse_talkgroup_json(json: &Value) -> Result<DmrTalkgroup, Box<dyn Error>> {
    let talkgroup = DmrTalkgroup {
        index: json["ID"].as_u64().unwrap_or(0) as usize,
        id: json["DMR ID"].as_u64().unwrap_or(0) as u32,
        name: json["Name"].as_str().unwrap_or("").to_string(),
        call_type: match json["Type"].as_str().unwrap_or("") {
            "Group" => DmrTalkgroupCallType::Group,
            "Private" => DmrTalkgroupCallType::Private,
            _ => return Err("Invalid talkgroup type".into()),
        },
        alert: false,
    };
    Ok(talkgroup)
}

fn parse_talkgroup_list_json(idx: usize,json: &Value, codeplug: &Codeplug) -> Result<DmrTalkgroupList, Box<dyn Error>> {
    let mut talkgroup_list = DmrTalkgroupList {
        index: idx,
        name: json["Name"].as_str().unwrap_or("").to_string(),
        talkgroups: Vec::new(),
    };

    if let Some(contact_indices) = json["Contacts"].as_array() {
        for contact_index in contact_indices {
            // get talkgroup by index and add it to the list
            let talkgroup = codeplug.talkgroups.iter().find(|tg| tg.index == contact_index.as_u64().unwrap_or(0) as usize);
            if let Some(talkgroup) = talkgroup {
                talkgroup_list.talkgroups.push(talkgroup.clone());
            }
        }
    }

    Ok(talkgroup_list)
}

fn parse_scan_list_json(index: usize, json: &Value) -> Result<ScanList, Box<dyn Error>> {
    let scan_list = ScanList {
        index: index,
        name: json["Name"].as_str().unwrap().to_string(),
        channels: Vec::new(),
    };

    Ok(scan_list)
}

fn parse_tone(type_str: &str, tone_str: &str) -> Option<Tone> {
    match type_str {
        "CTCSS" => Some(Tone::Ctcss(tone_str.parse::<f64>().ok()?)),
        "DCS" => Some(Tone::Dcs(format!("D{:>03}N", tone_str.parse::<u32>().ok()?))),
        "DCS Invert" => Some(Tone::Dcs(format!("D{:>03}I", tone_str.parse::<u32>().ok()?))),
        _ => None,
    }
}

fn parse_channel_json(channel_json: &Value, codeplug: &Codeplug) -> Option<Channel> {
    let mut channel = Channel::default();
    // mode-common fields
    channel.index = channel_json["ID"].as_u64().unwrap() as usize;
    channel.name = channel_json["Name"].as_str().unwrap().to_string();
    channel.frequency_rx = Frequency::from_hz(channel_json["Rx Freq"].as_u64().unwrap() as f64);
    channel.frequency_tx = Frequency::from_hz(channel_json["Tx Freq"].as_u64().unwrap() as f64);
    channel.power = match channel_json["Tx Power"].as_str().unwrap() {
        // @TODO this is specific to DB25-D, may need to be adjusted for other radios
        "LOW" => Power::Watts(5.0),
        "HIGH" => Power::Watts(20.0),
        _ => Power::Watts(1.0),
    };
    channel.rx_only = channel_json["Rx only"].as_str().unwrap() == "ON";
    channel.tx_permit = match channel_json["TX Policy"].as_str().unwrap() {
        "POLITE_TO_ALL" => Some(TxPermit::ChannelFree),
        "POLITE_TO_CC" => Some(TxPermit::ColorCodeSame), // @TODO verify this
        "IMPOLITE" => Some(TxPermit::Always),
        _ => None
    };
    // mode-specific fields
    channel.mode = match channel_json["Type"].as_str().unwrap() {
        "ANALOG" => ChannelMode::FM,
        "DIGITAL" => ChannelMode::DMR,
        _ => ChannelMode::AM,
    };
    if channel.mode == ChannelMode::FM {
        channel.fm = Some(FmChannel {
            bandwidth: Frequency::from_khz_str(channel_json["Bandwidth"].as_str().unwrap().strip_suffix("KHz").unwrap()).unwrap(),
            squelch: Squelch::Default,
            tone_rx: parse_tone(
                channel_json["Tone Type Rx"].as_str().unwrap_or(""),
                channel_json["Tone Rx"].as_str().unwrap_or("")),
            tone_tx: parse_tone(
                channel_json["Tone Type Tx"].as_str().unwrap_or(""),
                channel_json["Tone Tx"].as_str().unwrap_or("")),
        });
    } else if channel.mode == ChannelMode::DMR {
        let contact_id = channel_json["Default Contact ID"].as_u64().unwrap() as usize;
        let group_list_id = channel_json["Group call list"].as_u64().unwrap() as usize;
        channel.dmr = Some(DmrChannel {
            // @TODO FIXME cpeditor supports different Tx/Rx timeslots, for now we only parse the TX timeslot
            timeslot: channel_json["TS Tx"].as_str().unwrap().strip_prefix("TS").unwrap().parse::<u8>().unwrap(),
            // @TODO FIXME cpeditor supports different Tx/Rx color codes, for now we only parse the TX color code
            color_code: channel_json["TX CC"].as_u64().unwrap() as u8,
            talkgroup: if contact_id > 0 {
                // index into codeplug.talkgroups with contact_id
                codeplug.talkgroups.get(contact_id - 1).map(|tg| tg.name.clone())
            } else {
                None
            },
            talkgroup_list: if group_list_id > 0 {
                // index into codeplug.talkgroup_lists with group_list_id
                codeplug.talkgroup_lists.get(group_list_id - 1).map(|tgl| tgl.name.clone())
            } else {
                None
            },
            id_name: None,
        });
    }
    Some(channel)
}

fn parse_zone_json(zone_json: &Value, codeplug: &mut Codeplug) {
    let index = zone_json["ID"].as_u64().unwrap() as usize;
    let name = zone_json["Name"].as_str().unwrap_or("").to_string();
    let mut zone = Zone {
        index,
        name,
        channels: Vec::new(),
    };

    // add channels to channels first, then add to zone
    if let Some(channels_json) = zone_json["Channels"].as_array() {
        for channel_json in channels_json {
            if let Some(channel) = parse_channel_json(channel_json, codeplug) {
                zone.channels.push(channel.name.clone());
                codeplug.channels.push(channel);
            }
        }
    }

    codeplug.zones.push(zone);
}

pub fn read(opt: &Opt, input_path: &PathBuf) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug::default();
    codeplug.source = format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap());

    // check that the input path is a file
    if !input_path.is_file() {
        uprintln!(opt, Stderr, THEME.err, None, "You lied to me when you told me this was a file: {}", input_path.display());
        return Err("Bad input path".into());
    }

    uprintln!(opt, Stderr, None, 3, "Reading {}", input_path.display());
    // read file into JSON
    let mut file = File::open(input_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let json: Value = serde_json::from_str(&contents)?;

    // try to confirm that this looks like a valid codeplug
    if json["Device info"]["Model Number"] != "DB_25D" {
        uprintln!(opt, Stderr, THEME.err, None, "This does not look like a valid codeplug");
        return Err("Invalid codeplug".into());
    }

    // parse configuration
    codeplug.config = parse_config(opt, &json);

    // parse talkgroups
    if let Some(contacts) = json["Contacts"].as_array() {
        for contact in contacts {
            let talkgroup = parse_talkgroup_json(contact)?;
            codeplug.talkgroups.push(talkgroup);
        }
    }

    // parse talkgroup lists
    if let Some(rx_groups) = json["RX groups"].as_array() {
        for (ii, rx_group) in rx_groups.iter().enumerate() {
            let talkgroup_list = parse_talkgroup_list_json(ii + 1, rx_group, &codeplug)?;
            codeplug.talkgroup_lists.push(talkgroup_list);
        }
    }

    // parse scan list names so channels can reference them
    if let Some(scan_lists) = json["Scan lists"].as_array() {
        for (ii, scan_list) in scan_lists.iter().enumerate() {
            let scan_list = parse_scan_list_json(ii + 1, scan_list)?;
            codeplug.scanlists.push(scan_list);
        }
    }

    // cpeditor is "zone-first", so we parse zones and channels simultaneously
    if let Some(zones) = json["Zones"].as_array() {
        for zone in zones.iter() {
            parse_zone_json(zone, &mut codeplug);
        }
    }
    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

fn write_contacts(opt: &Opt, codeplug: &Codeplug) -> Result<Value, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());

    let mut contacts_json = json!([]);

    for (ii, talkgroup) in codeplug.talkgroups.iter().enumerate() {
        let contact = json!({
            "ID": ii + 1,
            "Name": talkgroup.name.clone(),
            "DMR ID": talkgroup.id,
            "Type": match talkgroup.call_type {
                DmrTalkgroupCallType::Group => "Group",
                DmrTalkgroupCallType::Private => "Private",
                _ => {
                    uprintln!(opt, Stderr, THEME.err, None, "Unsupported talkgroup type: {:?}", talkgroup.call_type);
                    continue;
                },
            },
        });
        contacts_json.as_array_mut().unwrap().push(contact);
    }

    Ok(contacts_json)
}

fn write_scanlists(opt: &Opt, codeplug: &Codeplug) -> Result<Value, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());

    let mut scanlists_json = json!([]);

    // build scanlists from zones
    for zone in codeplug.zones.iter() {
        let scanlist = json!({
            "Name": zone.name.clone(),
        });
        scanlists_json.as_array_mut().unwrap().push(scanlist);
    }

    Ok(scanlists_json)
}

fn write_rx_groups(opt: &Opt, codeplug: &Codeplug) -> Result<Value, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());

    let mut rx_groups_json = json!([]);

    for talkgroup_list in codeplug.talkgroup_lists.iter() {
        let rx_group = json!({
            "Name": talkgroup_list.name.clone(),
            "Contacts": [],
        });
        rx_groups_json.as_array_mut().unwrap().push(rx_group);
    }

    Ok(rx_groups_json)
}

fn write_channel(opt: &Opt, channel: &Channel) -> Result<Value, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    let mut channel_json = json!({
        "ID": channel.index + 1,
        "Type": match channel.mode {
            ChannelMode::FM => "ANALOG",
            ChannelMode::DMR => "DIGITAL",
            _ => return Err("Unsupported channel mode".into()),
        },
        "Name": channel.name.clone(),
        "Rx Freq": channel.frequency_rx.hz() as u64,
        "Tx Freq": channel.frequency_tx.hz() as u64,
        "Tx Power": match channel.power {
            Power::Watts(w) => {
                if w > 5.0 {
                    "HIGH"
                } else {
                    "LOW"
                }
            },
            Power::Default => "HIGH",
        },
        "Rx Only": if channel.rx_only {
            "ON"
        } else {
            "OFF"
        },
        "Alarm": "OFF",
        "PROMPT": "OFF",
        "PCT": "PATCS",
    });
    // mode specific fields
    match channel.mode {
        ChannelMode::FM => {
            // DMR fields are set to defaults
            channel_json["DMR Mode"] = json!("Double Slot");
            channel_json["TS Rx"] = json!("TS1");
            channel_json["TS Tx"] = json!("TS1");
            channel_json["RX CC"] = json!(1);
            channel_json["TX CC"] = json!(1);
            channel_json["MSG Type"] = json!("UNCONFIRMED");
            channel_json["TX Policy"] = json!("IMPOLITE"); // @TODO FIXME
            channel_json["Group call list"] = json!(0);
            channel_json["Scan List ID"] = json!(0); // @TODO FIXME
            channel_json["Default Contact ID"] = json!(0);
            channel_json["EAS"] = json!("OFF");
            channel_json["Bandwidth"] = json!("25KHz"); // @TODO FIXME
            channel_json["Tone Type Tx"] = json!("DCS"); // @TODO FIXME
            channel_json["Tone Tx"] = json!("17"); // @TODO FIXME
            channel_json["Tone Type Rx"] = json!("DCS"); // @TODO FIXME
            channel_json["Tone Rx"] = json!("17"); // @TODO FIXME
            channel_json["APRS Channel"] = json!(0);
            channel_json["Relay Monitor"] = json!("OFF");
            channel_json["Relay Mode"] = json!("OFF");
            channel_json["Encryption"] = json!(0);

        },
        ChannelMode::DMR => {
            channel_json["DMR Mode"] = json!("Double Slot");
            channel_json["TS Rx"] = json!("TS1");
            channel_json["TS Tx"] = json!("TS1");
            channel_json["RX CC"] = json!(1);
            channel_json["TX CC"] = json!(1);
            channel_json["MSG Type"] = json!("UNCONFIRMED");
            channel_json["TX Policy"] = json!("IMPOLITE"); // @TODO FIXME
            channel_json["Group call list"] = json!(0);
            channel_json["Scan List ID"] = json!(0); // @TODO FIXME
            channel_json["Default Contact ID"] = json!(0);
            channel_json["EAS"] = json!("OFF");
            channel_json["Bandwidth"] = json!("25KHz"); // @TODO FIXME
            channel_json["Tone Type Tx"] = json!("DCS"); // @TODO FIXME
            channel_json["Tone Tx"] = json!("17"); // @TODO FIXME
            channel_json["Tone Type Rx"] = json!("DCS"); // @TODO FIXME
            channel_json["Tone Rx"] = json!("17"); // @TODO FIXME
            channel_json["APRS Channel"] = json!(0);
            channel_json["Relay Monitor"] = json!("OFF");
            channel_json["Relay Mode"] = json!("OFF");
            channel_json["Encryption"] = json!(0);
        },
        _ => return Err("Unsupported channel mode".into()),
    }

    channel_json["TS Rx"] = json!("TS1");
    channel_json["TS Tx"] = json!("TS1");

    Ok(channel_json)
}

fn write_zones(opt: &Opt, codeplug: &Codeplug) -> Result<Value, Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());

    let mut zones_json = json!([]);

    for (ii, zone) in codeplug.zones.iter().enumerate() {
        let mut zone_json = json!({
            "ID": ii + 1,
            "Name": zone.name.clone(),
            "Channels": [],
        });
        for channel_name in zone.channels.iter() {
            // get channel by name
            let channel = codeplug.channels.iter().find(|c| c.name == *channel_name).unwrap();
            // convert channel to JSON
            let channel_json = write_channel(opt, channel)?;
            zone_json["Channels"].as_array_mut().unwrap().push(channel_json);
        }

        zones_json.as_array_mut().unwrap().push(zone_json);
    }

    Ok(zones_json)
}

pub fn write(opt: &Opt, codeplug: &Codeplug, output_path: &PathBuf, donor_path: &Option<PathBuf>) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, THEME.trace, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    // if the output path exists, complain
    if output_path.exists() {
        uprintln!(opt, Stderr, THEME.err, None, "Output path already exists: {}", output_path.display());
        return Err("Output path already exists".into());
    }

    // if donor path doesn't exist, complain
    if donor_path.is_none() {
        uprintln!(opt, Stderr, THEME.err, None, "Donor file not provided");
        return Err("Donor file required".into());
    }

    // load the donor JSON file
    let donor_str = fs::read_to_string(donor_path.as_ref().unwrap())?;
    let mut donor: Value = serde_json::from_str(&donor_str)?;

    // check that the donor file at least has a model that we recognize
    match donor["Device info"]["Model Number"].as_str().unwrap() {
        "DB_25D" => uprintln!(opt, Stderr, THEME.info, None, "Donor: Radioddity DB25-D"),
        _ => {
            uprintln!(opt, Stderr, THEME.err, None,"Unrecognized model: {}", donor["Device info"]["Model Number"].as_str().unwrap());
            return Err("Unrecognized model".into());
        }
    }

    // mangle the donor JSON
    donor["Contacts"] = write_contacts(opt, &codeplug)?;
    donor["Scan lists"] = write_scanlists(opt, &codeplug)?;
    donor["RX groups"] = write_rx_groups(opt, &codeplug)?;
    // zones contain channels
    donor["Zones"] = write_zones(opt, &codeplug)?;

    // write to output path
    fs::write(output_path, serde_json::to_string_pretty(&donor).unwrap().as_bytes())?;

    Ok(())
}
