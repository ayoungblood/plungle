// src/radios/alinco_dj-md5t.rs

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
use rust_decimal::prelude::*;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::*;
use crate::structures::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::FM, structures::ChannelMode::DMR];
        props.channels_max = 4000;
        props.channel_name_width_max = 16;
        props.zones_max = 250;
        props.zone_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// CSV Export Format
// Alinco DJ-MD5T CPS Version v1.13e
/* Files
 * 2ToneEncode.CSV
 * 5ToneEncode.CSV
 * AnalogAddressBook.CSV
 * Aprs.CSV
 * AutoRepeaterOffsetFrequencies.CSV
 * Channel.CSV
 * DigitalContactList.CSV
 * DTMFEncode.CSV
 * FM.CSV
 * HotKey_HotKey.CSV
 * HotKey_QuickCall.CSV
 * HotKey_State.CSV
 * PrefabricatedSMS.CSV
 * RadioIDList.CSV
 * ReceiveGroupCallList.CSV
 * ScanList.CSV
 * TalkGroups.CSV
 * Zone.CSV
 */

// Channel.CSV
// - No.: Channel index
// - Channel Name: Channel name
// - Receive Frequency: frequency in MHz
// - Transmit Frequency: frequency in MHz
// - Channel Type: [A-Analog, D-Digital]
// - Transmit Power: [Turbo, High, Mid, Low] but manual says "High: 5W, Middle: 2.5W, Low: 1W, Small: 0.2W"
// - Band Width: [12.5K, 25K]
// - CTCSS/DCS Decode: Off, or CTCSS/DCS frequency/code
// - CTCSS/DCS Encode: Off, or CTCSS/DCS frequency/code
// - Contact: DMR contact
// - Contact Call Type: [Group Call, ???]
// - Contact TG/DMR ID: DMR talkgroup ID
// - Radio ID: Radio ID name (not DMR ID), generally callsign
// - Busy Lock/TX Permit: [Off, Busy]
// - Squelch Mode: [Carrier, CTCSS/DCS], Carrier for digital channels
// - Optional Signal: Off
// - DTMF ID: 1
// - 2Tone ID: 1
// - 5Tone ID: 1
// - PTT ID: Off
// - Color Code: DMR color code, 0-15
// - Slot: DMR timeslot, [1, 2]
// - Scan List: None or Scan List name
// - Receive Group List: None or RX Group List name
// - TX Prohibit: [Off, On]
// - Reverse: [Off, On]
// - Simplex TDMA: [Off, ??]
// - TDMA Adaptive: [Off, ??]
// - Encryption Type: [Normal Encryption, ??]
// - Digital Encryption: [Off, ??]
// - Call Confirmation: [Off, ??]
// - Talk Around: [Off, ??]
// - Work Alone: [Off, ??]
// - Custom CTCSS: [251.1, ??]
// - 2TONE Decode: [0, ??]
// - Ranging: [Off, ??]
// - Through Mode: [Off, ??]

// RadioIDList.CSV
// - No.: radio ID index
// - Radio ID: radio ID
// - Name: radio ID name

// ReceiveGroupCallList.CSV
// - No.: talkgroup list index
// - Group Name: talkgroup list name
// - Contact: list of DMR talkgroup names, "|" separated
// - Contact TG/DMR ID: list of DMR talkgroup IDs, "|" separated

// ScanList.CSV
// - No: scan list index
// - Scan List Name: scan list name
// - Scan Channel Member: list of channel names, "|" separated
// - Scan Mode: [Off, ??]
// - Priority Channel Select: [Off, ??]
// - Priority Channel 1: [Off, ??]
// - Priority Channel 2: [Off, ??]
// - Revert Channel: [Selected, ??]
// - Look Back Time A[s]: default 2
// - Look Back Time B[s]: default 3
// - Dropout Delay Time[s]: default 3.1
// - Dwell Time[s]: default 3.1

// TalkGroups.CSV
// - No.: talkgroup index
// - Radio ID: DMR talkgroup ID
// - Name: DMR talkgroup name
// - Call Type: [Group Call, All Call, Private Call]
// - Call Alert: [None, ??]

// Zone.CSV
// - No.: zone index
// - Zone Name: zone name
// - Zone Channel Member: list of channel names, "|" separated
// - A Channel: name of selected channel in zone
// - B Channel: name of selected channel in zone

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

fn parse_talkgroup_record(record: &CsvRecord, opt: &Opt) -> Result<DmrTalkgroup, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    static TALKGROUP_INDEX: AtomicUsize = AtomicUsize::new(1);
    let talkgroup = DmrTalkgroup {
        index: TALKGROUP_INDEX.fetch_add(1, Ordering::Relaxed),
        id: record.get("Radio ID").unwrap().parse::<u32>()?,
        name: record.get("Name").unwrap().to_string(),
        call_type: match record.get("Call Type").unwrap().as_str() {
            "Group Call" => DmrTalkgroupCallType::Group,
            "Private Call" => DmrTalkgroupCallType::Private,
            "All Call" => DmrTalkgroupCallType::AllCall,
            _ => return Err(format!("Unrecognized call type: {}", record.get("Call Type").unwrap()).into()),
        },
        alert: false, // @TODO FIXME
    };

    Ok(talkgroup)
}

fn parse_talkgroup_list_record(record: &CsvRecord, codeplug: &Codeplug, opt: &Opt) -> Result<DmrTalkgroupList, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    static TALKGROUP_LIST_INDEX: AtomicUsize = AtomicUsize::new(1);
    let mut talkgroup_list = DmrTalkgroupList {
        index: TALKGROUP_LIST_INDEX.fetch_add(1, Ordering::Relaxed),
        name: record.get("Group Name").unwrap().to_string(),
        talkgroups: Vec::new(),
    };

    // Talkgroup names are stored as a list, separated by "|"
    let talkgroup_names: Vec<&str> = record.get("Contact").unwrap().split('|').collect();
    // Find the talkgroup by name
    for name in talkgroup_names {
        let talkgroup = codeplug.talkgroups.iter().find(|&t| t.name == name);
        match talkgroup {
            Some(t) => talkgroup_list.talkgroups.push(t.clone()),
            None => return Err(format!("Talkgroup not found: {}", name).into()),
        }
    }

    Ok(talkgroup_list)
}

// Convert a string into a TxPermit enum
fn parse_tx_permit(tx_permit: &str) -> Option<TxPermit> {
    match tx_permit {
        "Always" => Some(TxPermit::Always),
        "Busy" => Some(TxPermit::ChannelFree),
        _ => return None,
    }
}

// Convert a CTCSS/DCS string into a Tone struct
// Anytone stores CTCSS/DCS as follows:
// - "Off" for no tone
// - "100" or "141.3" for CTCSS frequency (decimal point may or may not be present)
// - "D023N" or "D023I" for DCS code (N for normal, I for inverted)
fn parse_tone(tone: &str) -> Option<Tone> {
    if tone == "Off" {
        return None;
    }
    // if string begins with D, it's DCS
    if tone.starts_with("D") {
        return Some(Tone::Dcs(tone.trim().to_string()));
    }
    return Some(Tone::Ctcss(tone.parse::<f64>().unwrap()));
}

// Convert the CSV channel hashmap into a Channel struct
fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let mut channel = Channel::default();

    channel.index = record.get("No.").unwrap().parse::<usize>()?;
    channel.name = record.get("Channel Name").unwrap().to_string();
    channel.mode = match record.get("Channel Type").unwrap().as_str() {
        "A-Analog" => ChannelMode::FM,
        "D-Digital" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel type: {}", record.get("Channel Type").unwrap()).into()),
    };
    channel.frequency_rx = Decimal::from_str(record.get("Receive Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.frequency_tx = Decimal::from_str(record.get("Transmit Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.rx_only = record.get("TX Prohibit").unwrap() == "On";
    channel.power = match record.get("Transmit Power").unwrap().as_str() {
        // @TODO manual disagrees with CPS, no idea what these values are
        "Turbo" => Power::Watts(7.0), // 7W
        "High" => Power::Watts(5.0), // 5W
        "Mid" => Power::Watts(2.5), // 2.5W
        "Low" => Power::Watts(1.0), // 1W
        _ => return Err(format!("Unrecognized power: {}", record.get("Transmit Power").unwrap()).into()),
    };
    channel.tx_permit = parse_tx_permit(record.get("Busy Lock/TX Permit").unwrap().as_str());
    if channel.mode == ChannelMode::FM { // FM specific fields
        channel.fm = Some(FmChannel {
            bandwidth: match record.get("Band Width").unwrap().as_str() {
                "12.5K" => Decimal::from_str("12.5").unwrap() * Decimal::new(1_000, 0),
                "25K" => Decimal::from_str("25.0").unwrap() * Decimal::new(1_000, 0),
                _ => return Err(format!("Unrecognized bandwidth: {}", record.get("Band Width").unwrap()).into()),
            },
            squelch: Squelch::Default,
            tone_rx: parse_tone(record.get("CTCSS/DCS Decode").unwrap().as_str()),
            tone_tx: parse_tone(record.get("CTCSS/DCS Encode").unwrap().as_str()),
        });
    } else if channel.mode == ChannelMode::DMR { // DMR specific fields
        channel.dmr = Some(DmrChannel {
            timeslot: record.get("Slot").unwrap().parse::<u8>()?,
            color_code: record.get("Color Code").unwrap().parse::<u8>()?,
            // digital channels will always have Contact set (name of a talkgroup/group or private call),
            // and optionally will have Receive Group List set (name of a talkgroup list) or "None" if no list
            talkgroup: record.get("Contact").map(|s| s.to_string()),
            talkgroup_list: if record.get("Receive Group List").unwrap() == "None" {
                None
            } else {
                Some(record.get("Receive Group List").unwrap().to_string())
            },
            id_name: Some(record.get("Radio ID").unwrap().to_string()),
        })
    } else {
        return Err("Unparsed channel mode".into());
    }

    Ok(channel)
}

// Convert the CSV zone hashmap into a Zone struct
fn parse_zone_record(csv_zone: &CsvRecord, codeplug: &Codeplug, opt: &Opt) -> Result<Zone, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", csv_zone);
    static ZONE_INDEX: AtomicUsize = AtomicUsize::new(1);
    let mut zone = Zone {
        index: ZONE_INDEX.fetch_add(1, Ordering::Relaxed),
        name: csv_zone.get("Zone Name").unwrap().to_string(),
        channels: Vec::new(),
    };

    // Channels are stored as a list of names, separated by "|"
    let channel_names: Vec<&str> = csv_zone.get("Zone Channel Member").unwrap().split('|').collect();
    for name in channel_names {
        // find the channel by name in the codeplug
        let channel = codeplug.channels.iter().find(|&c| c.name == name);
        match channel {
            Some(c) => zone.channels.push(c.name.clone()),
            None => return Err(format!("Channel not found: {}", name).into()),
        }
    }

    Ok(zone)
}

// Convert the CSV DMR ID hashmap into a DMRId struct
fn parse_dmr_id_record(csv_dmr_id: &CsvRecord, opt: &Opt) -> Result<DmrId, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", csv_dmr_id);
    let dmr_id = DmrId {
        id: csv_dmr_id.get("Radio ID").unwrap().parse::<u32>()?,
        name: csv_dmr_id.get("Name").unwrap().to_string(),
    };

    Ok(dmr_id)
}

pub fn read(input_path: &PathBuf, opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug::default();
    codeplug.source = format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap());

    // check that the input path is a directory
    if !input_path.is_dir() {
        uprintln!(opt, Stderr, Color::Red, None, "You lied to me when you told me this was a directory: {}", input_path.display());
        return Err("Bad input path".into());
    }

    // check for TalkGroups.CSV
    let mut talkgroups_path: PathBuf = input_path.clone();
    talkgroups_path.push("TalkGroups.CSV");
    // if this file doesn't exist, no problem, we just don't have any talkgroups
    if talkgroups_path.exists() {
        uprintln!(opt, Stderr, None, 3, "Reading {}", talkgroups_path.display());
        let mut reader = csv::Reader::from_path(talkgroups_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to DmrTalkgroup struct
            let talkgroup = parse_talkgroup_record(&record, &opt)?;
            // append to codeplug.talkgroups
            codeplug.talkgroups.push(talkgroup);
        }
    }

    // Check for ReceiveGroupCallList.CSV
    let mut talkgroup_lists_path: PathBuf = input_path.clone();
    talkgroup_lists_path.push("ReceiveGroupCallList.CSV");
    // if this file doesn't exist, no problem, we just don't have any talkgroup lists
    // also, no point in reading this if we don't have any talkgroups
    if talkgroup_lists_path.exists() && !codeplug.talkgroups.is_empty() {
        uprintln!(opt, Stderr, None, 3, "Reading {}", talkgroup_lists_path.display());
        let mut reader = csv::Reader::from_path(talkgroup_lists_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to DmrTalkgroupList struct
            let talkgroup_list = parse_talkgroup_list_record(&record, &codeplug, &opt)?;
            // append to codeplug.talkgroup_lists
            codeplug.talkgroup_lists.push(talkgroup_list);
        }
    }

    // Check for Channel.CSV
    let mut channels_path: PathBuf = input_path.clone();
    channels_path.push("Channel.CSV");
    if !channels_path.exists() {
        return Err("Channel.CSV not found".into());
    } else {
        uprintln!(opt, Stderr, None, 3, "Reading {}", channels_path.display());
        let mut reader = csv::Reader::from_path(channels_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Channel struct
            let channel = parse_channel_record(&record, &opt)?;
            // Alinco DJ-MD5T stores VFO A/B at 4001/4002, skip these
            if (channel.index == 4001 && channel.name == "Channel VFO A") ||
               (channel.index == 4002 && channel.name == "Channel VFO B") {
                continue;
            }
            // append to codeplug.channels
            codeplug.channels.push(channel);
        }
    }

    // Check for Zone.CSV
    let mut zones_path: PathBuf = input_path.clone();
    zones_path.push("Zone.CSV");
    // if Zone.CSV doesn't exist, no problem, we just don't have any zones
    if zones_path.exists() {
        uprintln!(opt, Stderr, None, 3, "Reading {}", zones_path.display());
        let mut reader = csv::Reader::from_path(zones_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Zone struct
            let zone = parse_zone_record(&record, &codeplug, &opt)?;
            // append to codeplug.zones
            codeplug.zones.push(zone);
        }
    }

    // Check for RadioIDList.CSV
    let mut radio_id_list_path: PathBuf = input_path.clone();
    radio_id_list_path.push("RadioIDList.CSV");
    // if RadioIDList.CSV doesn't exist, no problem, we just don't have any radio IDs
    if radio_id_list_path.exists() {
        let mut reader = csv::Reader::from_path(radio_id_list_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to DmrId struct
            let dmr_id = parse_dmr_id_record(&record, &opt)?;
            // append to codeplug.config.dmr_configuration.id_list
            if codeplug.config.is_none() {
                codeplug.config = Some(Configuration {
                    dmr_configuration: Some(DmrConfiguration {
                        id_list: Vec::new(),
                    }),
                });
            }
            codeplug.config.as_mut().unwrap().dmr_configuration.as_mut().unwrap().id_list.push(dmr_id);
        }
    }

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

pub fn write_talkgroups(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always) // Alinco CPS expects all fields to be quoted
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Radio ID",
        "Name",
        "Call Type",
        "Call Alert",
    ])?;

    for (ii, talkgroup) in codeplug.talkgroups.iter().enumerate() {
        uprintln!(opt, Stderr, None, 4, "Writing talkgroup {:width$}: {:?}", talkgroup.id, talkgroup.name, width = 4);
        writer.write_record(&[
            format!("{}", ii + 1), // No.
            talkgroup.id.to_string(), // Radio ID
            talkgroup.name.clone(), // Name
            match talkgroup.call_type {
                DmrTalkgroupCallType::Group => "Group Call".to_string(),
                DmrTalkgroupCallType::Private => "Private Call".to_string(),
                DmrTalkgroupCallType::AllCall => "All Call".to_string(),
            }, // Call Type
            "None".to_string(), // Call Alert
        ])?;
    }

    writer.flush()?;

    Ok(())
}

pub fn write_talkgroup_lists(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always) // Alinco CPS expects all fields to be quoted
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Group Name",
        "Contact",
        "Contact TG/DMR ID",
    ])?;

    for (ii, talkgroup_list) in codeplug.talkgroup_lists.iter().enumerate() {
        uprintln!(opt, Stderr, None, 4, "Writing talkgroup list {:width$}: {}", ii + 1, talkgroup_list.name, width = 3);
        let mut contact = String::new();
        let mut contact_id = String::new();
        for (jj, talkgroup) in talkgroup_list.talkgroups.iter().enumerate() {
            if jj > 0 {
                contact.push_str("|");
                contact_id.push_str("|");
            }
            contact.push_str(&talkgroup.name);
            contact_id.push_str(&format!("\"{}\"", talkgroup.id));
        }
        writer.write_record(&[
            format!("{}", ii + 1), // No.
            talkgroup_list.name.clone(), // Group Name
            contact, // Contact
            contact_id, // Contact TG/DMR ID
        ])?;
    }

    writer.flush()?;

    // absolutely horrific mungling
    // the CPS expects contact IDs to be double-quoted, separated by |, and then all fields are quoted
    // this results in a double-double quote, which the csv library turns into triple quotes, but we need double quotes
    // open the file and replace double-double-quotes with a single double quote
    let mut contents = fs::read_to_string(path)?;
    contents = contents.replace("\"\"", "\"");
    fs::write(path, contents)?;

    Ok(())
}

fn write_power(power: &Power) -> String {
    match power {
        Power::Default => "High".to_string(),
        Power::Watts(w) if *w >= 7.0 => "Turbo".to_string(),
        Power::Watts(w) if *w >= 5.0 => "High".to_string(),
        Power::Watts(w) if *w >= 2.5 => "Mid".to_string(),
        Power::Watts(w) if *w >= 1.0 => "Low".to_string(),
        _ => "Low".to_string(),
    }
}

fn get_talkgroup_type_string(talkgroup: &DmrTalkgroup) -> String {
    match talkgroup.call_type {
        DmrTalkgroupCallType::Group => "Group Call".to_string(),
        DmrTalkgroupCallType::Private => "Private Call".to_string(),
        DmrTalkgroupCallType::AllCall => "All Call".to_string(),
    }
}

// get a tuple with the talkgroup name, type, and id
fn get_contact_tuple(channel: &Channel, codeplug: &Codeplug) -> (String, String, String) {
    if channel.dmr.is_some() && channel.dmr.as_ref().unwrap().talkgroup.is_some() {
        // if the channel has a talkgroup set, use that
        let talkgroup_name = channel.dmr.as_ref().unwrap().talkgroup.as_ref().unwrap();
        // find the talkgroup by name in the codeplug
        let talkgroup = codeplug.talkgroups.iter().find(|&t| t.name == *talkgroup_name).unwrap();
        let call_type = get_talkgroup_type_string(&talkgroup);
        return (talkgroup.name.clone(), call_type.to_string(), talkgroup.id.to_string());
    } else if channel.dmr.is_some() && channel.dmr.as_ref().unwrap().talkgroup_list.is_some() {
        // if the channel has a talkgroup list, pick the first contact in the talkgroup list
        if let Some(talkgroup_list_name) = &channel.dmr.as_ref().unwrap().talkgroup_list {
            // find the talkgroup list by name in the codeplug
            if let Some(talkgroup) = codeplug.talkgroup_lists.iter().find(|&t| t.name == *talkgroup_list_name) {
                let talkgroup = &talkgroup.talkgroups[0];
                let call_type = get_talkgroup_type_string(&talkgroup);
                return (talkgroup.name.clone(), call_type.to_string(), talkgroup.id.to_string());
            }
        }
    } else {
        // if the channel has neither a talkgroup or a talkgroup list, use the first talkgroup
        // this is used for analog channels to match the CPS behaviour
        if !codeplug.talkgroups.is_empty() {
            let talkgroup = &codeplug.talkgroups[0];
            let call_type = get_talkgroup_type_string(&talkgroup);
            return (talkgroup.name.clone(), call_type.to_string(), talkgroup.id.to_string());
        }
    }
    // if everything fails, return empty strings
    ("".to_string(), "".to_string(), "".to_string())
}

fn write_radio_id(channel: &Channel, codeplug: &Codeplug) -> String {
    // if id_name  exists, use it, otherwise return the first radio ID in the codeplug
    if let Some(dmr) = &channel.dmr {
        if let Some(id_name) = &dmr.id_name {
            return id_name.to_string();
        }
    } else if let Some(config) = &codeplug.config {
        if let Some(dmr_config) = &config.dmr_configuration {
            if !dmr_config.id_list.is_empty() {
                return dmr_config.id_list[0].name.clone();
            }
        }
    }
    // if all else fails, return an empty string
    "".to_string()
}

fn write_tx_permit(channel: &Channel) -> String {
    let tx_permit = match &channel.tx_permit {
        Some(TxPermit::Always) => "Always".to_string(),
        Some(TxPermit::ChannelFree) => "Busy".to_string(),
        _ => "Off".to_string(),

    };
    tx_permit
}

// scan list needs to be set in the channel
// right now, we build the scan list from the zone
// so just pick the first zone that contains the channel, and set that as the scan list (if it exists)
fn write_scan_list(channel: &Channel, codeplug: &Codeplug) -> String {
    for zone in &codeplug.zones {
        if zone.channels.contains(&channel.name) {
            return zone.name.clone();
        }
    }
    "None".to_string()
}

pub fn write_channels(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Channel Name",
        "Receive Frequency",
        "Transmit Frequency",
        "Channel Type",
        "Transmit Power",
        "Band Width",
        "CTCSS/DCS Decode",
        "CTCSS/DCS Encode",
        "Contact",
        "Contact Call Type",
        "Contact TG/DMR ID",
        "Radio ID",
        "Busy Lock/TX Permit",
        "Squelch Mode",
        "Optional Signal",
        "DTMF ID",
        "2Tone ID",
        "5Tone ID",
        "PTT ID",
        "Color Code",
        "Slot",
        "Scan List",
        "Receive Group List",
        "TX Prohibit",
        "Reverse",
        "Simplex TDMA",
        "TDMA Adaptive",
        "Encryption Type",
        "Digital Encryption",
        "Call Confirmation",
        "Talk Around",
        "Work Alone",
        "Custom CTCSS",
        "2TONE Decode",
        "Ranging",
        "Through Mode",
    ])?;

    for channel in &codeplug.channels {
        uprintln!(opt, Stderr, None, 4, "Writing channel {:width$}: {}", channel.index, channel.name, width = get_props().channel_index_width);
        uprintln!(opt, Stderr, None, 4, "    {:?}", channel);

        let contact = get_contact_tuple(&channel, &codeplug);
        if channel.mode == ChannelMode::FM {
            writer.write_record(&[
                channel.index.to_string(), // No.
                channel.name.clone(), // Channel Name
                format!("{:0.5}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Receive Frequency
                format!("{:0.5}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Transmit Frequency
                "A-Analog".to_string(), // Channel Type
                write_power(&channel.power), // Transmit Power
                match channel.fm.clone().unwrap().bandwidth.to_f64().unwrap() {
                    12_500.0 => "12.5K".to_string(),
                    25_000.0 => "25K".to_string(),
                    _ => return Err("Unrecognized bandwidth".into()),
                }, // Band Width
                if let Some(tone) = &channel.fm.as_ref().unwrap().tone_rx {
                    match tone {
                        Tone::Ctcss(ctcss) => format!("{:0.1}", ctcss),
                        Tone::Dcs(dcs) => dcs.clone(),
                    }
                } else {
                    "Off".to_string()
                }, // CTCSS/DCS Decode
                if let Some(tone) = &channel.fm.as_ref().unwrap().tone_tx {
                    match tone {
                        Tone::Ctcss(ctcss) => format!("{:0.1}", ctcss),
                        Tone::Dcs(dcs) => dcs.clone(),
                    }
                } else {
                    "Off".to_string()
                }, // CTCSS/DCS Encode
                contact.0, // Contact
                contact.1, // Contact Call Type
                contact.2, // Contact TG/DMR ID
                write_radio_id(&channel, &codeplug), // Radio ID
                write_tx_permit(&channel), // Busy Lock/TX Permit
                if channel.fm.as_ref().unwrap().tone_rx.is_some() {
                    "CTCSS/DCS".to_string()
                } else {
                    "Carrier".to_string()
                }, // Squelch Mode
                "Off".to_string(), // Optional Signal
                "1".to_string(), // DTMF ID
                "1".to_string(), // 2Tone ID
                "1".to_string(), // 5Tone ID
                "Off".to_string(), // PTT ID
                "1".to_string(), // Color Code
                "1".to_string(), // Slot
                write_scan_list(&channel, &codeplug), // Scan List
                "None".to_string(), // Receive Group List
                if channel.rx_only {
                    "On".to_string()
                } else {
                    "Off".to_string()
                }, // TX Prohibit
                "Off".to_string(), // Reverse
                "Off".to_string(), // Simplex TDMA
                "Off".to_string(), // TDMA Adaptive
                "Normal Encryption".to_string(), // Encryption Type
                "Off".to_string(), // Digital Encryption
                "Off".to_string(), // Call Confirmation
                "Off".to_string(), // Talk Around
                "Off".to_string(), // Work Alone
                "251.1".to_string(), // Custom CTCSS
                "0".to_string(), // 2TONE Decode
                "Off".to_string(), // Ranging
                "Off".to_string(), // Through Mode
            ])?;
        } else if channel.mode == ChannelMode::DMR {
            writer.write_record(&[
                channel.index.to_string(), // No.
                channel.name.clone(), // Channel Name
                format!("{:0.5}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Receive Frequency
                format!("{:0.5}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Transmit Frequency
                "D-Digital".to_string(), // Channel Type
                write_power(&channel.power), // Transmit Power
                "12.5K".to_string(), // Band Width
                "Off".to_string(), // CTCSS/DCS Decode
                "Off".to_string(), // CTCSS/DCS Encode
                contact.0, // Contact
                contact.1, // Contact Call Type
                contact.2, // Contact TG/DMR ID
                write_radio_id(&channel, &codeplug), // Radio ID
                write_tx_permit(&channel), // Busy Lock/TX Permit
                "Carrier".to_string(), // Squelch Mode
                "Off".to_string(), // Optional Signal
                "1".to_string(), // DTMF ID
                "1".to_string(), // 2Tone ID
                "1".to_string(), // 5Tone ID
                "Off".to_string(), // PTT ID
                channel.dmr.as_ref().unwrap().color_code.to_string(), // Color Code
                channel.dmr.as_ref().unwrap().timeslot.to_string(), // Slot
                write_scan_list(&channel, &codeplug), // Scan List
                if let Some(talkgroup_list) = &channel.dmr.as_ref().unwrap().talkgroup_list {
                    talkgroup_list.clone()
                } else {
                    "None".to_string()
                }, // Receive Group List
                if channel.rx_only {
                    "On".to_string()
                } else {
                    "Off".to_string()
                }, // TX Prohibit
                "Off".to_string(), // Reverse
                "Off".to_string(), // Simplex TDMA
                "Off".to_string(), // TDMA Adaptive
                "Normal Encryption".to_string(), // Encryption Type
                "Off".to_string(), // Digital Encryption
                "Off".to_string(), // Call Confirmation
                "Off".to_string(), // Talk Around
                "Off".to_string(), // Work Alone
                "251.1".to_string(), // Custom CTCSS
                "0".to_string(), // 2TONE Decode
                "Off".to_string(), // Ranging
                "Off".to_string(), // Through Mode
            ])?;
        } else {
            uprintln!(opt, Stderr, Color::Red, None, "Unsupported channel mode: index = {}, mode = {:?}", channel.index, channel.mode);
        }
    }

    writer.flush()?;

    Ok(())
}

pub fn write_zones(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Zone Name",
        "Zone Channel Member",
        "A Channel",
        "B Channel",
    ])?;

    for (ii, zone) in codeplug.zones.iter().enumerate() {
        uprintln!(opt, Stderr, None, 4, "Writing zone {:width$}: {}", ii + 1, zone.name, width = get_props().zone_index_width);

        let mut channel_names = String::new();
        for (jj, name) in zone.channels.iter().enumerate() {
            if jj > 0 {
                channel_names.push_str("|");
            }
            let channel = codeplug.channels.iter().find(|&c| c.name == *name).unwrap();
            channel_names.push_str(&channel.name);
        }
        // get the first channel in the zone
        let a_channel = codeplug.channels.iter().find(|&c| c.name == zone.channels[0]).unwrap();
        // get the second channel in the zone, if it exists
        let b_channel = codeplug.channels.iter().find(|&c| c.name == *zone.channels.get(1).unwrap_or(&zone.channels[0])).unwrap();
        writer.write_record(&[
            format!("{}", ii + 1), // No.
            zone.name.clone(), // Zone Name
            channel_names, // Zone Channel Member
            a_channel.name.clone(), // A Channel
            b_channel.name.clone(), // B Channel
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_scanlists(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Scan List Name",
        "Scan Channel Member",
        "Scan Mode",
        "Priority Channel Select",
        "Priority Channel 1",
        "Priority Channel 2",
        "Revert Channel",
        "Look Back Time A[s]",
        "Look Back Time B[s]",
        "Dropout Delay Time[s]",
        "Dwell Time[s]",
    ])?;

    for (ii, zone) in codeplug.zones.iter().enumerate() {
        uprintln!(opt, Stderr, None, 4, "Writing scan list {:width$}: {}", ii + 1, zone.name, width = get_props().zone_index_width);
        let mut channel_names = String::new();
        for (jj, name) in zone.channels.iter().enumerate() {
            if jj > 0 {
                channel_names.push_str("|");
            }
            let channel = codeplug.channels.iter().find(|&c| c.name == *name).unwrap();
            channel_names.push_str(&channel.name);
        }

        writer.write_record(&[
            format!("{}", ii + 1), // No.
            zone.name.clone(), // Scan List Name
            channel_names, // Scan Channel Member
            "Off".to_string(), // Scan Mode
            "Off".to_string(), // Priority Channel Select
            "Off".to_string(), // Priority Channel 1
            "Off".to_string(), // Priority Channel 2
            "Selected".to_string(), // Revert Channel
            "2.0".to_string(), // Look Back Time A[s]
            "3.0".to_string(), // Look Back Time B[s]
            "3.1".to_string(), // Dropout Delay Time[s]
            "3.1".to_string(), // Dwell Time[s]
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_radio_id_list(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always) // Anytone CPS expects all fields to be quoted
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Radio ID",
        "Name",
    ])?;

    for (ii, dmr_id) in codeplug.config.as_ref().unwrap().dmr_configuration.as_ref().unwrap().id_list.iter().enumerate() {
        uprintln!(opt, Stderr, None, 4, "Writing radio ID {:width$}: {}", dmr_id.id, dmr_id.name, width = 8);
        writer.write_record(&[
            format!("{}", ii + 1), // No.
            dmr_id.id.to_string(), // Radio ID
            dmr_id.name.clone(), // Name
        ])?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write(codeplug: &Codeplug, output_path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    // if the output path exists, check if it is an empty directory
    // if it does not exist, create it
    if output_path.exists() {
        if output_path.is_dir() {
            // check if the directory is empty
            let dir_entries = std::fs::read_dir(output_path)?;
            if dir_entries.count() > 0 {
                uprintln!(opt, Stderr, Color::Red, None, "Output path exists and is not empty, not overwriting!");
                return Err("Bad output path".into());
            }
        }
    } else {
        // if it does not exist, create it
        std::fs::create_dir_all(output_path)?;
    }
    if fs::metadata(output_path)?.permissions().readonly() {
        uprintln!(opt, Stderr, Color::Red, None, "Output path is read-only, cannot write!");
        return Err("Bad output path".into());
    }

    // write TalkGroups.CSV
    let mut talkgroups_path: PathBuf = output_path.clone();
    talkgroups_path.push("TalkGroups.CSV");
    if codeplug.talkgroups.len() > 0 {
        write_talkgroups(&codeplug, &talkgroups_path, &opt)?;
    }

    // write ReceiveGroupCallList.CSV
    let mut talkgroup_lists_path: PathBuf = output_path.clone();
    talkgroup_lists_path.push("ReceiveGroupCallList.CSV");
    if codeplug.talkgroup_lists.len() > 0 {
        write_talkgroup_lists(&codeplug, &talkgroup_lists_path, &opt)?;
    }

    // write Channel.CSV
    let mut channels_path: PathBuf = output_path.clone();
    channels_path.push("Channel.CSV");
    write_channels(&codeplug, &channels_path, &opt)?;

    // write Zone.CSV
    let mut zones_path: PathBuf = output_path.clone();
    zones_path.push("Zone.CSV");
    if codeplug.zones.len() > 0 {
        write_zones(&codeplug, &zones_path, &opt)?;
    }

    // write ScanList.CSV
    // Copy zones to scan lists since we don't support scan lists yet @TODO
    let mut scanlists_path: PathBuf = output_path.clone();
    scanlists_path.push("ScanList.CSV");
    if codeplug.zones.len() > 0 {
        write_scanlists(&codeplug, &scanlists_path, &opt)?;
    }

    // write to RadioIDList.CSV
    let mut radio_id_list_path: PathBuf = output_path.clone();
    radio_id_list_path.push("RadioIDList.CSV");
    if let Some(config) = &codeplug.config {
        if let Some(dmr_config) = &config.dmr_configuration {
            if dmr_config.id_list.len() > 0 {
                write_radio_id_list(codeplug, &radio_id_list_path, opt)?;
            }
        }
    }

    Ok(())
}

