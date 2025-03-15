// src/radios/opengd77_rt3s.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

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
        props.channels_max = 1024;
        props.channel_name_width_max = 16;
        props.zones_max = 68;
        props.zone_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props.zone_index_width = (props.zones_max as f64).log10().ceil() as usize;
        props
    })
}

// CSV Export Format
// OpenGD77 CPS Version R2024.09.13.02
/* Files
 * APRS.csv
 * Channels.csv
 * Contacts.csv
 * DTMF.csv
 * TG_Lists.csv
 * Zones.csv
 */

// Channels.csv
// - Channel Number: channel index
// - Channel Name: @TODO how many characters
// - Channel Type: [Analogue,Digital]
// - Rx Frequency: frequency in MHz, zero-padded to five decimal places
// - Tx Frequency: frequency in MHz, zero-padded to five decimal places
// - Bandwidth (kHz): [12.5,25], blank for Digital
// - Colour Code: [0-15], blank for Analogue
// - Timeslot: [1,2], blank for Analogue
// - Contact: talkgroup name, blank for Analogue, None for when TG List below is set
// - TG List: talkgroup list name, blank for Analogue, None for when Contact above is set
// - DMR ID: None
// - TS1_TA_Tx: timeslot 1 talker alias, [Off, ???] @TODO
// - TS2_TA_Tx ID: timeslot 2 talker alias, [Off, ???] @TODO
// - RX Tone: None, CTCSS frequency in Hz, or DCS code (DnnnN or DnnnI), blank for Digital
// - TX Tone: None, CTCSS frequency in Hz, or DCS code (DnnnN or DnnnI), blank for Digital
// - Squelch: blank for Digital, [Disabled,Open,Closed,5%..95%] (default is Disabled)
// - Power: [Master,P1,P2,P3,P4,P5,P6,P7,P8,P9,-W+] corresponding to [default,50mW,250mW,500mW,750mW,1W,2W,3W,4W,5W,-W+]
//   - OpenGD77 uses -W+ for user configurable power which may be ~6W on the RT3S at max PA drive, but may also be lower than 50mW if configured
// - Rx Only: [No, Yes]
// - Zone Skip: [No, Yes]
// - All Skip: [No, Yes]
// - TOT: timeout timer, [15-495s,0 for infinite], set to 60s for Rx Only, default 0 (infinite)
// - VOX: Off, ??? @TODO
// - No Beep: No, ??? @TODO
// - No Eco: No, ??? @TODO
// - APRS: None, ??? @TODO
// - Latitude: ??? @TODO
// - Longitude: ??? @TODO
// - Use location: [No, Yes]

// Contacts.csv
// - Contact Name: talkgroup name
// - ID: talkgroup ID
// - ID Type: [Group,Private,AllCall]
// - TS Override: [Disabled, 1, 2]

// TG_Lists.csv
// - TG List Name: talkgroup list name
// - Contact1..Contact32: talkgroup name, blank if not used

// Zones.csv
// - Zone Name: zone name
// - Channel1..Channel80: channel name, blank if not used

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

pub fn parse_talkgroup_record(record: &CsvRecord, opt: &Opt) -> Result<DmrTalkgroup, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let talkgroup = DmrTalkgroup {
        id: record.get("ID").unwrap().parse()?,
        name: record.get("Contact Name").unwrap().to_string(),
        call_type: match record.get("ID Type").unwrap().as_str() {
            "Group" => DmrTalkgroupCallType::Group,
            "Private" => DmrTalkgroupCallType::Private,
            "AllCall" => DmrTalkgroupCallType::AllCall,
            _ => return Err(format!("Unrecognized call type: {}", record.get("Call Type").unwrap()).into()),
        },
        alert: false, // not supported
    };
    Ok(talkgroup)
}

pub fn parse_talkgroup_list_record(record: &CsvRecord, codeplug: &Codeplug, opt: &Opt) -> Result<DmrTalkgroupList, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let mut talkgroup_list = DmrTalkgroupList {
        name: record.get("TG List Name").unwrap().to_string(),
        talkgroups: Vec::new(),
    };
    // iterate over the contacts in the CSV record, up to 32 (do not use for (k,v) in .. because it doesn't care about order)
    for ii in 1..=32 {
        let key = format!("Contact{}", ii);
        let value = record.get(&key).unwrap();
        if value != "" {
            // find the talkgroup in codeplug.talkgroups
            let talkgroup = codeplug.talkgroups.iter().find(|&x| x.name == *value);
            if let Some(tg) = talkgroup {
                talkgroup_list.talkgroups.push(tg.clone());
            } else {
                uprintln!(opt, Stderr, Color::Yellow, None, "Talkgroup not found: {}", value);
            }
        }
    }
    Ok(talkgroup_list)
}

// Convert a power string into a Power struct
// OpenGD77 stores power as follows:
// - "Master" for default power
// - "P1".."P9" for power levels
// - "-W+" for user configurable power
fn parse_power(power: &str) -> Power {
    return match power {
        "Master" => Power::Default,
        "P1" => Power::Watts(0.05), //  50mW
        "P2" => Power::Watts(0.25), // 250mW
        "P3" => Power::Watts(0.50), // 500mW
        "P4" => Power::Watts(0.75), // 750mW
        "P5" => Power::Watts(1.00), // 1W
        "P6" => Power::Watts(2.00), // 2W
        "P7" => Power::Watts(3.00), // 3W
        "P8" => Power::Watts(4.00), // 4W
        "P9" => Power::Watts(5.00), // 5W
        "-W+" => Power::Watts(6.0), // user configurable, just assume 6W for now
        _ => panic!("Unrecognized power level: {}", power),
    }
}

// Convert a CTCSS/DCS string into a Tone struct
// OpenGD77 stores CTCSS/DCS as follows:
// - "None" for no tone
// - "100" or "141.3" for CTCSS frequency (decimal point may or may not be present)
// - "D023N" or "D754I" for DCS code (N for normal, I for inverted)
fn parse_tone(tone: &str) -> Option<Tone> {
    if tone == "Off" || tone == "None" {
        return None;
    }
    // if string begins with D, it's DCS
    if tone.starts_with("D") {
        return Some(Tone::Dcs(tone.trim().to_string()));
    }
    return Some(Tone::Ctcss(tone.trim().parse::<f64>().unwrap()));
}

// Convert a squelch string into a Squelch struct
// OpenGD77 stores squelch as follows:
// - "Disabled" for default squelch (set by menu)
// - "Open" for squelch open (0%)
// - "Closed" for squelch closed (100%)
// - "5%".."95%" for squelch level
fn parse_squelch(squelch: &str) -> Squelch {
    match squelch {
        "Disabled" => Squelch::Default,
        "Open" => Squelch::Percent(0),
        "Closed" => Squelch::Percent(100),
        _ => {
            Squelch::Percent(squelch.trim_end_matches('%').parse().unwrap())
        },
    }
}

pub fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let mut channel = Channel::default();

    // shared fields
    channel.index = record.get("Channel Number").unwrap().parse::<usize>()?;
    channel.name = record.get("Channel Name").unwrap().to_string();
    channel.mode = match record.get("Channel Type").unwrap().as_str() {
        "Analogue" => ChannelMode::FM,
        "Digital" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel type: {}", record.get("Channel Type").unwrap()).into()),
    };
    channel.frequency_rx = Decimal::from_str(record.get("Rx Frequency").unwrap().trim())? * Decimal::new(1_000_000, 0);
    channel.frequency_tx = Decimal::from_str(record.get("Tx Frequency").unwrap().trim())? * Decimal::new(1_000_000, 0);
    channel.rx_only = record.get("Rx Only").unwrap() == "Yes";
    if record.get("TOT").unwrap() == "0" {
        channel.tx_tot = Timeout::Infinite;
    } else {
        channel.tx_tot = Timeout::Seconds(record.get("TOT").unwrap().parse::<u32>()?);
    }
    channel.power = parse_power(record.get("Power").unwrap().as_str());
    if record.get("Zone Skip").unwrap() == "Yes" || record.get("All Skip").unwrap() == "Yes" {
        channel.scan = Some(Scan {
            zone_skip: record.get("Zone Skip").unwrap() == "Yes",
            all_skip: record.get("All Skip").unwrap() == "Yes",
        });
    }

    if channel.mode == ChannelMode::FM { // FM specific fields
        channel.fm = Some(FmChannel {
            bandwidth: Decimal::from_str(record.get("Bandwidth (kHz)").unwrap())? * Decimal::new(1_000, 0),
            squelch: parse_squelch(record.get("Squelch").unwrap().as_str()),
            tone_rx: parse_tone(record.get("RX Tone").unwrap().as_str()),
            tone_tx: parse_tone(record.get("TX Tone").unwrap().as_str()),
        });
    } else if channel.mode == ChannelMode::DMR { // DMR specific fields
        channel.dmr = Some(DmrChannel {
            timeslot: record.get("Timeslot").unwrap().parse::<u8>()?,
            color_code: record.get("Colour Code").unwrap().parse::<u8>()?,
            // digital channels will have either a talkgroup or a talkgroup list
            talkgroup: if record.get("Contact").unwrap() == "None" {
                None
            } else {
                Some(record.get("Contact").unwrap().to_string())
            },
            talkgroup_list: if record.get("TG List").unwrap() == "None" {
                None
            } else {
                Some(record.get("TG List").unwrap().to_string())
            },
            id_name: None,
        });
    }
    Ok(channel)
}

pub fn parse_zone_record(record: &CsvRecord, codeplug: &Codeplug, opt: &Opt) -> Result<Zone, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    static ZONE_INDEX: AtomicUsize = AtomicUsize::new(1);
    let mut zone = Zone {
        index: ZONE_INDEX.fetch_add(1, Ordering::Relaxed),
        name: record.get("Zone Name").unwrap().to_string(),
        channels: Vec::new(),
    };
    // iterate over the channels in the CSV record, up to 80 (do not use for (k,v) in .. because it doesn't care about order)
    for ii in 1..=80 {
        let key = format!("Channel{}", ii);
        let value = record.get(&key).unwrap();
        if value != "" {
            // find the channel in codeplug.channels
            let channel = codeplug.channels.iter().find(|&x| x.name == *value);
            if let Some(ch) = channel {
                zone.channels.push(ch.name.clone());
            } else {
                uprintln!(opt, Stderr, Color::Yellow, None, "Channel not found: {}", value);
            }
        }
    }
    Ok(zone)
}

pub fn read(input_path: &PathBuf, opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    let mut codeplug = Codeplug {
        channels: Vec::new(),
        zones: Vec::new(),
        talkgroups: Vec::new(),
        talkgroup_lists: Vec::new(),
        config: None,
        source: format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap()),
    };

    // check that the input path is a directory
    if !input_path.is_dir() {
        uprintln!(opt, Stderr, Color::Red, None, "You lied to me when you told me this was a directory: {}", input_path.display());
        return Err("Bad input path".into());
    }

    // Check for Contacts.csv
    let mut talkgroups_path: PathBuf = input_path.clone();
    talkgroups_path.push("Contacts.csv");
    // if Contacts.csv doesn't exist, no problem, we just don't have any talkgroups
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

    // Check for TG_Lists.csv
    let mut talkgroup_lists_path: PathBuf = input_path.clone();
    talkgroup_lists_path.push("TG_Lists.csv");
    // if TG_Lists.csv doesn't exist, no problem, we just don't have any talkgroup lists
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

    // check for Channels.csv
    let mut channels_path: PathBuf = input_path.clone();
    channels_path.push("Channels.csv");
    if !channels_path.exists() {
        return Err("Channels.csv not found".into());
    } else {
        uprintln!(opt, Stderr, None, 3, "Reading {}", channels_path.display());
        let mut reader = csv::Reader::from_path(channels_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Channel struct
            let channel = parse_channel_record(&record, &opt)?;
            // append to codeplug.channels
            codeplug.channels.push(channel);
        }
    }

    // check for Zones.csv
    let mut zones_path: PathBuf = input_path.clone();
    zones_path.push("Zones.csv");
    // if Zones.csv doesn't exist, no problem, we just don't have any zones
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
    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

pub fn write_talkgroups(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "Contact Name",
        "ID",
        "ID Type",
        "TS Override",
    ])?;

    for talkgroup in &codeplug.talkgroups {
        uprintln!(opt, Stderr, None, 4, "Writing talkgroup: {}", talkgroup.name);
        writer.write_record(&[
            talkgroup.name.clone(), // Contact Name
            talkgroup.id.to_string(), // ID
            match talkgroup.call_type {
                DmrTalkgroupCallType::Group => "Group",
                DmrTalkgroupCallType::Private => "Private",
                DmrTalkgroupCallType::AllCall => "AllCall",
            }.to_string(), // ID Type
            "Disabled".to_string(), // TS Override
        ])?;
    }

    writer.flush()?;

    Ok(())
}

pub fn write_talkgroup_lists(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    let mut header: Vec<String> = vec!["TG List Name".to_string()];
    for ii in 1..=32 {
        header.push(format!("Contact{}", ii));
    }
    writer.write_record(&header)?;

    for talkgroup_list in &codeplug.talkgroup_lists {
        uprintln!(opt, Stderr, None, 4, "Writing talkgroup list: {}", talkgroup_list.name);
        let mut record = vec![talkgroup_list.name.clone()];
        for tg in &talkgroup_list.talkgroups {
            record.push(tg.name.clone());
        }
        // pad the record with empty strings to 32 elements
        while record.len() < 33 {
            record.push("".to_string());
        }
        writer.write_record(&record)?;
    }

    writer.flush()?;

    Ok(())
}

fn write_tone(tone: &Option<Tone>) -> String {
    match tone {
        Some(tone) => {
            match tone {
                Tone::Ctcss(f) => format!("{:0.1}", f),
                Tone::Dcs(d) => d.to_string(),
            }
        },
        None => "None".to_string(),
    }
}

fn write_squelch(squelch: &Squelch) -> String {
    match squelch { // 0 is Open, 100 is Closed
        Squelch::Default => "Disabled".to_string(),
        Squelch::Percent(p) if *p <    5 => format!("Open"),
        Squelch::Percent(p) if *p == 100 => format!("Closed"),
        Squelch::Percent(p) => format!("{}%", (p / 5) * 5),
    }
}

fn write_power(power: &Power) -> String {
    match power {
        Power::Default => "Master".to_string(),
        Power::Watts(w) => {
            if *w >= 6.0 {
                "-W+".to_string()
            } else if *w >= 5.0 {
                "P9".to_string()
            } else if *w >= 4.0 {
                "P8".to_string()
            } else if *w >= 3.0 {
                "P7".to_string()
            } else if *w >= 2.0 {
                "P6".to_string()
            } else if *w >= 1.0 {
                "P5".to_string()
            } else if *w >= 0.75 {
                "P4".to_string()
            } else if *w >= 0.50 {
                "P3".to_string()
            } else if *w >= 0.25 {
                "P2".to_string()
            } else {
                "P1".to_string()
            }
        }
    }
}

fn write_tx_tot(tx_tot: &Timeout) -> String {
    match tx_tot {
        Timeout::Default => "0".to_string(),
        Timeout::Seconds(s) => s.to_string(),
        Timeout::Infinite => "0".to_string(),
    }
}

pub fn write_channels(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "Channel Number",
        "Channel Name",
        "Channel Type",
        "Rx Frequency",
        "Tx Frequency",
        "Bandwidth (kHz)",
        "Colour Code",
        "Timeslot",
        "Contact",
        "TG List",
        "DMR ID",
        "TS1_TA_Tx",
        "TS2_TA_Tx ID",
        "RX Tone",
        "TX Tone",
        "Squelch",
        "Power",
        "Rx Only",
        "Zone Skip",
        "All Skip",
        "TOT",
        "VOX",
        "No Beep",
        "No Eco",
        "APRS",
        "Latitude",
        "Longitude",
        "Use location",
    ])?;

    for channel in &codeplug.channels {
        uprintln!(opt, Stderr, None, 4, "Writing channel {:width$}: {}", channel.index, channel.name, width = get_props().channel_index_width);
        if channel.mode == ChannelMode::FM {
            writer.write_record(&[
                channel.index.to_string(), // Channel Number
                channel.name.clone(), // Channel Name
                "Analogue".to_string(), // Channel Type
                // put a tab in front to prevent Excel from mangling it
                format!("\t{:0.5}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Rx Frequency
                format!("\t{:0.5}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Tx Frequency
                (channel.fm.as_ref().unwrap().bandwidth / Decimal::new(1_000, 0)).to_string(), // Bandwidth
                "".to_string(), // Colour Code
                "".to_string(), // Timeslot
                "".to_string(), // Contact
                "".to_string(), // TG List
                "".to_string(), // DMR ID
                "".to_string(), // TS1_TA_Tx
                "".to_string(), // TS2_TA_Tx ID
                write_tone(&channel.fm.as_ref().unwrap().tone_rx), // RX Tone
                write_tone(&channel.fm.as_ref().unwrap().tone_tx), // TX Tone
                write_squelch(&channel.fm.as_ref().unwrap().squelch), // Squelch
                write_power(&channel.power), // Power
                if channel.rx_only { "Yes".to_string() } else { "No".to_string() },
                if channel.scan.is_some() && channel.scan.as_ref().unwrap().zone_skip { "Yes".to_string() } else { "No".to_string() }, // Zone Skip
                if channel.scan.is_some() && channel.scan.as_ref().unwrap().all_skip { "Yes".to_string() } else { "No".to_string() }, // All Skip
                write_tx_tot(&channel.tx_tot), // TOT
                "Off".to_string(), // VOX
                "No".to_string(), // No Beep
                "No".to_string(), // No Eco
                "None".to_string(), // APRS
                "0".to_string(), // Latitude
                "0".to_string(), // Longitude
                "No".to_string(), // Use Location
            ])?;
        } else if channel.mode == ChannelMode::DMR {
            writer.write_record(&[
                channel.index.to_string(), // Channel Number
                channel.name.clone(), // Channel Name
                "Digital".to_string(), // Channel Type
                // put a tab in front to prevent Excel from mangling it
                format!("\t{:0.5}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Rx Frequency
                format!("\t{:0.5}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Tx Frequency
                "".to_string(), // Bandwidth
                channel.dmr.as_ref().unwrap().color_code.to_string(), // Colour Code
                channel.dmr.as_ref().unwrap().timeslot.to_string(), // Timeslot
                if channel.dmr.as_ref().unwrap().talkgroup.is_some() {
                    channel.dmr.as_ref().unwrap().talkgroup.as_ref().unwrap().to_string()
                } else {
                    "None".to_string()
                }, // Contact
                if channel.dmr.as_ref().unwrap().talkgroup_list.is_some() {
                    channel.dmr.as_ref().unwrap().talkgroup_list.as_ref().unwrap().to_string()
                } else {
                    "None".to_string()
                }, // TG List
                "None".to_string(), // DMR ID
                "Off".to_string(), // TS1_TA_Tx
                "Off".to_string(), // TS2_TA_Tx ID
                "".to_string(), // Rx Tone
                "".to_string(), // Tx Tone
                "".to_string(), // Squelch
                write_power(&channel.power), // Power
                if channel.rx_only { "Yes".to_string() } else { "No".to_string() },
                if channel.scan.is_some() && channel.scan.as_ref().unwrap().zone_skip { "Yes".to_string() } else { "No".to_string() }, // Zone Skip
                if channel.scan.is_some() && channel.scan.as_ref().unwrap().all_skip { "Yes".to_string() } else { "No".to_string() }, // All Skip
                write_tx_tot(&channel.tx_tot), // TOT
                "Off".to_string(), // VOX
                "No".to_string(), // No Beep
                "No".to_string(), // No Eco
                "None".to_string(), // APRS
                "0".to_string(), // Latitude
                "0".to_string(), // Longitude
                "No".to_string(), // Use location
            ])?;
        } else {
            uprintln!(opt, Stderr, Color::Yellow, None, "Unsupported channel mode: index = {}, mode = {:?}", channel.index, channel.mode);
        }
    }

    writer.flush()?;

    Ok(())
}

fn write_zones(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    let mut header: Vec<String> = vec!["Zone Name".to_string()];
    for ii in 1..=80 {
        header.push(format!("Channel{}", ii));
    }
    writer.write_record(&header)?;

    for zone in &codeplug.zones {
        uprintln!(opt, Stderr, None, 4, "Writing zone: {}", zone.name);
        let mut record = vec![zone.name.clone()];
        for ch in &zone.channels {
            record.push(ch.clone());
        }
        // pad the record with empty strings to 80 elements
        while record.len() < 81 {
            record.push("".to_string());
        }
        writer.write_record(&record)?;
    }

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

    // write to Contacts.csv
    let mut talkgroups_path: PathBuf = output_path.clone();
    talkgroups_path.push("Contacts.csv");
    if codeplug.talkgroups.len() > 0 {
        write_talkgroups(&codeplug, &talkgroups_path, opt)?;
    }

    // write to TG_Lists.csv
    let mut talkgroup_lists_path: PathBuf = output_path.clone();
    talkgroup_lists_path.push("TG_Lists.csv");
    if codeplug.talkgroup_lists.len() > 0 {
        write_talkgroup_lists(&codeplug, &talkgroup_lists_path, opt)?;
    }

    // write to Channels.csv
    let mut channels_path: PathBuf = output_path.clone();
    channels_path.push("Channels.csv");
    write_channels(&codeplug, &channels_path, opt)?;

    // write to Zones.csv
    let mut zones_path: PathBuf = output_path.clone();
    zones_path.push("Zones.csv");
    if codeplug.zones.len() > 0 {
        write_zones(&codeplug, &zones_path, opt)?;
    }

    Ok(())
}
