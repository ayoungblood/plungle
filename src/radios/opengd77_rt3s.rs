// src/radios/opengd77_rt3s.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use rust_decimal::prelude::*;
use std::sync::OnceLock;

use crate::*;
use crate::structures::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.channels_max = 1024;
        props.channel_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
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
// - Bandwidth: [12.5,25], blank for Digital
// - Colour Code: [0-15], blank for Analogue
// - Timeslot: [1,2], blank for Analogue
// - Contact: talkgroup name, blank for Analogue, None for when TG List below is set
// - TG List: talkgroup list name, blank for Analogue, None for when Contact above is set
// - DMR ID: None
// - TS1_TA_Tx: timeslot 1 talker alias, [Off, ???] @TODO
// - TS2_TA_Tx ID: timeslot 2 talker alias, [Off, ???] @TODO
// - Rx Tone: None, CTCSS frequency in Hz, or DCS code (DnnnN or DnnnI), blank for Digital
// - Tx Tone: None, CTCSS frequency in Hz, or DCS code (DnnnN or DnnnI), blank for Digital
// - Squelch: blank for Digital, [Disabled,Open,Closed,5%..95%] (default is Disabled)
// - Power: [Master,P1,P2,P3,P4,P5,P6,P7,P8,P9,-W+] corresponding to [default,50mW,250mW,500mW,750mW,1W,2W,3W,4W,5W,+W-]
//   - OpenGD77 uses +W- for user configurable power which may be ~6W on the RT3S at max PA drive, but may also be lower than 50mW if configured
// - Rx Only: [No, Yes]
// - Zone Skip: [No, Yes]
// - All Skip: [No, Yes]
// - TOT: timeout timer in seconds, 0-??, 0 for disabled
// - VOX: Off, ??? @TODO
// - No Beep: No, ??? @TODO
// - No Eco: No, ??? @TODO
// - APRS: None, ??? @TODO
// - Latitude: ??? @TODO
// - Longitude: ??? @TODO
// - Use Location: Yes, ??? @TODO

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

pub fn parse_talkgroup_record(record: &CsvRecord) -> Result<DmrTalkgroup, Box<dyn Error>> {
    let talkgroup = DmrTalkgroup {
        id: record.get("ID").unwrap().parse()?,
        name: record.get("Contact Name").unwrap().to_string(),
        call_type: match record.get("ID Type").unwrap().as_str() {
            "Group" => DmrTalkgroupCallType::Group,
            "Private" => DmrTalkgroupCallType::Private,
            "AllCall" => DmrTalkgroupCallType::AllCall,
            _ => return Err(format!("Unrecognized call type: {}", record.get("Call Type").unwrap()).into()),
        },
    };
    Ok(talkgroup)
}

pub fn parse_talkgroup_list_record(record: &CsvRecord, codeplug: &Codeplug) -> Result<DmrTalkgroupList, Box<dyn Error>> {
    let mut talkgroup_list = DmrTalkgroupList {
        name: record.get("TG List Name").unwrap().to_string(),
        talkgroups: Vec::new(),
    };
    // iterate over the contacts in the CSV record
    for (key, value) in record {
        if key.starts_with("Contact") {
            if value != "" {
                // find the talkgroup in codeplug.talkgroups
                let talkgroup = codeplug.talkgroups.iter().find(|&x| x.name == *value);
                if let Some(tg) = talkgroup {
                    talkgroup_list.talkgroups.push(tg.clone());
                } else {
                    cprintln!(ANSI_C_YLW, "Talkgroup not found: {}", value);
                }
            }
        }
    }
    Ok(talkgroup_list)
}

// Convert a CTCSS/DCS string into a Tone struct
// OpenGD77 stores CTCSS/DCS as follows:
// - "None" for no tone
// - "100" or "141.3" for CTCSS frequency (decimal point may or may not be present)
// - "D023N" or "D754I" for DCS code (N for normal, I for inverted)
fn parse_tone(tone: &str) -> Option<Tone> {
    if tone == "None" {
        return None;
    }
    // if string begins with D, it's a DCS code
    if tone.starts_with("D") {
        return Some(Tone {
            mode: ToneMode::DCS,
            ctcss: None,
            dcs: Some(tone.to_string()),
        });
    }
    Some(Tone {
        mode: ToneMode::CTCSS,
        ctcss: Some(Decimal::from_str(tone).unwrap()),
        dcs: None,
    })
}

pub fn parse_channel_record(record: &CsvRecord) -> Result<Channel, Box<dyn Error>> {
    let mut channel = Channel {
        index: 0,
        name: String::new(),
        mode: ChannelMode::AM, // Default mode
        frequency_rx: Decimal::new(0,0),
        frequency_tx: Decimal::new(0,0),
        rx_only: false,
        power: Decimal::new(0,0),
        fm: None,
        dmr: None,
    };

    // shared fields
    channel.index = record.get("Channel Number").unwrap().parse::<u32>()?;
    channel.name = record.get("Channel Name").unwrap().to_string();
    channel.mode = match record.get("Channel Type").unwrap().as_str() {
        "Analogue" => ChannelMode::FM,
        "Digital" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel type: {}", record.get("Channel Type").unwrap()).into()),
    };
    channel.frequency_rx = Decimal::from_str(record.get("Rx Frequency").unwrap().trim())? * Decimal::new(1_000_000, 0);
    channel.frequency_tx = Decimal::from_str(record.get("Tx Frequency").unwrap().trim())? * Decimal::new(1_000_000, 0);
    channel.rx_only = record.get("Rx Only").unwrap() == "Yes";
    channel.power = match record.get("Power").unwrap().as_str() {
        "Master" => Decimal::new(5, 0),
        "P1" => Decimal::new(50,0) / Decimal::new(1_000,0), // 50mW
        "P2" => Decimal::new(250,0) / Decimal::new(1_000,0), // 250mW
        "P3" => Decimal::new(500,0) / Decimal::new(1_000,0), // 500mW
        "P4" => Decimal::new(750,0) / Decimal::new(1_000,0), // 750mW
        "P5" => Decimal::new(1,0),
        "P6" => Decimal::new(2,0),
        "P7" => Decimal::new(3,0),
        "P8" => Decimal::new(4,0),
        "P9" => Decimal::new(5,0),
        "-W+" => Decimal::new(6,0), // @TODO maybe this should throw a warning?
        _ => return Err(format!("Unrecognized power level: {}", record.get("Power").unwrap()).into()),
    };

    if channel.mode == ChannelMode::FM { // FM specific fields
        channel.fm = Some(FmChannel {
            bandwidth: Decimal::from_str(record.get("Bandwidth (kHz)").unwrap())? * Decimal::new(1_000, 0),
            squelch_level: 0, // @TODO
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
        });
    }
    Ok(channel)
}

pub fn parse_zone_record(record: &CsvRecord, codeplug: &Codeplug) -> Result<Zone, Box<dyn Error>> {
    let mut zone = Zone {
        name: record.get("Zone Name").unwrap().to_string(),
        channels: Vec::new(),
    };
    // iterate over the channels in the CSV record
    for (key, value) in record {
        if key.starts_with("Channel") {
            if value != "" {
                // find the channel in codeplug.channels
                let channel = codeplug.channels.iter().find(|&x| x.name == *value);
                if let Some(ch) = channel {
                    zone.channels.push(ch.name.clone());
                } else {
                    cprintln!(ANSI_C_YLW, "Channel not found: {}", value);
                }
            }
        }
    }
    Ok(zone)
}

pub fn read(opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    dprintln!(opt.verbose, 4, "{:?}", get_props());

    let mut codeplug = Codeplug {
        channels: Vec::new(),
        zones: Vec::new(),
        talkgroups: Vec::new(),
        talkgroup_lists: Vec::new(),
        config: None,
    };

    // check that the input path is a directory
    let input_path = match &opt.input {
        Some(path) => {
            if path.is_dir() {
                path
            } else {
                cprintln!(ANSI_C_RED, "You lied to me when you told me this was a directory: {}", path.display());
                return Err("Bad input path".into());
            }
        }
        None => return Err("Bad input path".into()),
    };

    // Check for Contacts.csv
    let mut talkgroups_path: PathBuf = input_path.clone();
    talkgroups_path.push("Contacts.csv");
    // if Contacts.csv doesn't exist, no problem, we just don't have any talkgroups
    if talkgroups_path.exists() {
        dprintln!(opt.verbose, 3, "Reading {}", talkgroups_path.display());
        let mut reader = csv::Reader::from_path(talkgroups_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to DmrTalkgroup struct
            let talkgroup = parse_talkgroup_record(&record)?;
            // append to codeplug.talkgroups
            codeplug.talkgroups.push(talkgroup);
        }
    }

    // Check for TG_Lists.csv
    let mut talkgroup_lists_path: PathBuf = input_path.clone();
    talkgroup_lists_path.push("TG_Lists.csv");
    // if TG_Lists.csv doesn't exist, no problem, we just don't have any talkgroup lists
    if talkgroup_lists_path.exists() {
        dprintln!(opt.verbose, 3, "Reading {}", talkgroup_lists_path.display());
        let mut reader = csv::Reader::from_path(talkgroup_lists_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to DmrTalkgroupList struct
            let talkgroup_list = parse_talkgroup_list_record(&record, &codeplug)?;
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
        dprintln!(opt.verbose, 3, "Reading {}", channels_path.display());
        let mut reader = csv::Reader::from_path(channels_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Channel struct
            let channel = parse_channel_record(&record)?;
            // append to codeplug.channels
            codeplug.channels.push(channel);
        }
    }

    // check for Zones.csv
    let mut zones_path: PathBuf = input_path.clone();
    zones_path.push("Zones.csv");
    // if Zones.csv doesn't exist, no problem, we just don't have any zones
    if zones_path.exists() {
        dprintln!(opt.verbose, 3, "Reading {}", zones_path.display());
        let mut reader = csv::Reader::from_path(zones_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Zone struct
            let zone = parse_zone_record(&record, &codeplug)?;
            // append to codeplug.zones
            codeplug.zones.push(zone);
        }
    }
    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

pub fn write(codeplug: &Codeplug, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    dprintln!(opt.verbose, 4, "{:?}", get_props());

    // if the output path exists, check if it is an empty directory
    // if it does not exist, create it
    if let Some(output_path) = &opt.output {
        if output_path.exists() {
            if output_path.is_dir() {
                // check if the directory is empty
                let dir_entries = std::fs::read_dir(output_path)?;
                if dir_entries.count() > 0 {
                    cprintln!(ANSI_C_RED, "Output path exists and is not empty, not overwriting!");
                    return Err("Bad output path".into());
                }
            }
        } else {
            // if it does not exist, create it
            std::fs::create_dir_all(output_path)?;
        }
        if fs::metadata(output_path)?.permissions().readonly() {
            cprintln!(ANSI_C_RED, "Output path is read-only, cannot write!");
            return Err("Bad output path".into());
        }
    }

    // write to Channels.csv
    let mut channels_path: PathBuf = opt.output.as_ref().unwrap().clone();
    channels_path.push("Channels.csv");
    dprintln!(opt.verbose, 3, "Writing {}", channels_path.display());
    let mut writer = csv::Writer::from_path(channels_path)?;

    // write the header
    writer.write_record(&[
        "Channel Number",
        "Channel Name",
        "Channel Type",
        "Rx Frequency",
        "Tx Frequency",
        "Bandwidth",
        "Colour Code",
        "Timeslot",
        "Contact",
        "TG List",
        "DMR ID",
        "TS1_TA_Tx",
        "TS2_TA_Tx ID",
        "Rx Tone",
        "Tx Tone",
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
        "Use Location",
    ])?;

    for channel in &codeplug.channels {
        dprintln!(opt.verbose, 4, "Writing channel {:width$}: {}", channel.index, channel.name, width = get_props().channel_index_width);
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
                "123".to_string(), // Rx Tone
                "123".to_string(), // Tx Tone
                "Disabled".to_string(), // Squelch
                "Master".to_string(), // Power
                if channel.rx_only { "Yes".to_string() } else { "No".to_string() },
                "No".to_string(), // Zone Skip
                "No".to_string(), // All Skip
                "0".to_string(), // TOT
                "Off".to_string(), // VOX
                "No".to_string(), // No Beep
                "No".to_string(), // No Eco
                "None".to_string(), // APRS
                "0".to_string(), // Latitude
                "0".to_string(), // Longitude
                "Yes".to_string(), // Use Location
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
                "Master".to_string(), // Power
                if channel.rx_only { "Yes".to_string() } else { "No".to_string() },
                "No".to_string(), // Zone Skip
                "No".to_string(), // All Skip
                "0".to_string(), // TOT
                "Off".to_string(), // VOX
                "No".to_string(), // No Beep
                "No".to_string(), // No Eco
                "None".to_string(), // APRS
                "0".to_string(), // Latitude
                "0".to_string(), // Longitude
                "Yes".to_string(), // Use Location
            ])?;
        } else {
            cprintln!(ANSI_C_YLW, "Unsupported channel mode: index = {}, mode = {:?}", channel.index, channel.mode);
        }
    }

    writer.flush()?;

    Ok(())
}
