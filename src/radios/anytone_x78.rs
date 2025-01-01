// src/radios/anytone_x78.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

use std::error::Error;
use std::collections::HashMap;
use crate::Opt;
use rust_decimal::prelude::*;
use std::path::PathBuf;

use crate::structures::{ChannelMode, ToneMode, Tone, FM, DMR, Channel, Zone, DmrId, DmrConfiguration, Configuration, Codeplug};

use crate::*;

// CSV Export Format:
// Channel.CSV
// - No.: Channel Index
// - Channel Name: 16 characters?
// - Receive Frequency: frequency in MHz
// - Transmit Frequency: frequency in MHz
// - Channel Type: [A-Analog, D-Digital]
// - Transmit Power: [Turbo, High, Mid, Low], corresponding to ~7W, 5W, 2.5W, 1W
// - Band Width: [12.5K, 25K]
// - CTCSS/DCS Decode: Off, or CTCSS/DCS frequency/code
// - CTCSS/DCS Encode: Off, or CTCSS/DCS frequency/code
// - Contact: DMR contact
// - Contact Call Type: [Group Call, ???]
// - Contact TG/DMR ID: DMR talkgroup ID
// - Radio ID: Radio ID name (not DMR ID), generally callsign
// - Busy Lock/TX Permit: [Off, Always, ???]
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
// - PTT Prohibit: [Off, On]
// - Reverse: [Off, On]
// - Simplex TDMA: [Off, ??]
// - Slot Suit: [Off, ??]
// - AES Digital Encryption: Normal Encryption
// - Digital Encryption Type: [Off, ???]
// - Call Confirmation: [Off, ???]
// - Talk Around(Simplex): [Off, ???]
// - Work Alone: [Off, ???]
// - Custom CTCSS: 251.1 or custom frequency
// - 2TONE Decode: 0
// - Ranging: [Off, ???]
// - Through Mode: [Off, ???]
// - APRS RX: [Off, ???]
// - Analog APRS PTT Mode: [Off, ???]
// - Digital APRS PTT Mode: [Off, ???]
// - APRS Report Type: [Off, ???]
// - Digital APRS Report Channel: 1
// - Correct Frequency[Hz]: 0
// - SMS Confirmation: [Off, ???]
// - Exclude channel from roaming: [0, 1]
// - DMR MODE: 0
// - DataACK Disable: 0
// - R5toneBot: 0
// - R5ToneEot: 0
// - Auto Scan: 0
// - Ana Aprs Mute: 0
// - Send Talker Alias: 0
// - AnaAprsTxPath: 0
// - ARC4: 0
// - ex_emg_kind: 0
// TalkGroups.CSV
// - No.: DMR talkgroup index
// - Radio ID: DMR talkgroup ID
// - Name: DMR talkgroup name (@TODO length??)
// - Call Type: [Group Call, ???]
// - Call Alert: [None, ???]
// ReceiveGroupCallList.CSV
// - No.: talkgroup list index
// - Group Name: DMR talkgroup list name
// - Contact: list of DMR talkgroup names, "|" separated
// - Contact TG/DMR ID: list of DMR talkgroup IDs, "|" separated
// ScanList.CSV
// - No.: scan list index
// - Scan List Name: scan list name
// - Scan Channel Member: list of channel names, "|" separated
// - Scan Channel Member RX Frequency: list of channel RX frequencies in MHz, "|" separated
// - Scan Channel Member TX Frequency: list of channel TX frequencies in MHz, "|" separated
// - Scan Mode: [Off, ???]
// - Priority Channel Select: [Off, ???]
// - Priority Channel 1: [Off, ???]
// - Priority Channel 1 RX Frequency: [blank, ???]
// - Priority Channel 1 TX Frequency: [blank, ???]
// - Priority Channel 2: [Off, ???]
// - Priority Channel 2 RX Frequency: [blank, ???]
// - Priority Channel 2 TX Frequency: [blank, ???]
// - Revert Channel: [Selected, ???]
// - Look Back Time A[s]: default 2
// - Look Back Time B[s]: default 3
// - Dropout Delay Time[s]: default 3.1
// - Priority Sample Time[s]: default 3.1
// Zone.CSV
// - No.: zone index
// - Zone Name: zone name
// - Zone Channel Member: list of channel names, "|" separated
// - Zone Channel Member RX Frequency: list of channel RX frequencies in MHz, "|" separated
// - Zone Channel Member TX Frequency: list of channel TX frequencies in MHz, "|" separated
// - A Channel: name of selected channel in zone
// - A Channel RX Frequency: RX frequency in MHz of selected channel in zone
// - A Channel TX Frequency: TX frequency in MHz of selected channel in zone
// - B Channel: name of selected channel in zone
// - B Channel RX Frequency: RX frequency in MHz of selected channel in zone
// - B Channel TX Frequency: TX frequency in MHz of selected channel in zone
// - Zone Hide: [0, ???]
// RadioIDList.CSV
// - No.: radio ID index
// - Radio ID: radio ID
// - Name: radio ID name

type CsvRecord = HashMap<String, String>;

pub fn read(opt: &Opt) -> Result<Codeplug, Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());

    let mut codeplug = Codeplug {
        channels: Vec::new(),
        zones: Vec::new(),
        lists: Vec::new(),
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

    // Check for Channel.CSV
    let mut channel_path: PathBuf = input_path.clone();
    channel_path.push("Channel.CSV");
    if !channel_path.exists() {
        return Err("Channel.CSV not found".into());
    } else {
        dprintln!(opt.verbose, 3, "Reading {}", channel_path.display());
        let mut reader = csv::Reader::from_path(channel_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Channel struct
            let channel = parse_channel_record(&record)?;
            // append to codeplug.channels
            codeplug.channels.push(channel);
        }
    }

    // Check for Zone.CSV
    let mut zone_path: PathBuf = input_path.clone();
    zone_path.push("Zone.CSV");
    if !zone_path.exists() {
        return Err("Zone.CSV not found".into());
    } else {
        dprintln!(opt.verbose, 3, "Reading {}", zone_path.display());
        let mut reader = csv::Reader::from_path(zone_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Zone struct
            let zone = parse_zone_record(&record, &codeplug)?;
            // append to codeplug.zones
            codeplug.zones.push(zone);
        }
    }

    // Check for RadioIDList.CSV
    let mut radio_id_list_path: PathBuf = input_path.clone();
    radio_id_list_path.push("RadioIDList.CSV");
    // if this file doesn't exist, no problem, we just don't set the radio ID list
    if radio_id_list_path.exists() {
        dprintln!(opt.verbose, 3, "Reading {}", radio_id_list_path.display());
        let mut reader = csv::Reader::from_path(radio_id_list_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to DmrId struct
            let dmr_id = parse_dmr_id_record(&record)?;
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

// Convert the CSV channel hashmap into a Channel struct
fn parse_channel_record(csv_channel: &CsvRecord) -> Result<Channel, Box<dyn Error>> {
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

    channel.index = csv_channel.get("No.").unwrap().parse::<u32>()?;
    channel.name = csv_channel.get("Channel Name").unwrap().to_string();
    channel.mode = match csv_channel.get("Channel Type").unwrap().as_str() {
        "A-Analog" => ChannelMode::FM,
        "D-Digital" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel type: {}", csv_channel.get("Channel Type").unwrap()).into()),
    };
    channel.frequency_rx = Decimal::from_str(csv_channel.get("Receive Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.frequency_tx = Decimal::from_str(csv_channel.get("Transmit Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.rx_only = csv_channel.get("PTT Prohibit").unwrap() == "On";
    channel.power = match csv_channel.get("Transmit Power").unwrap().as_str() {
        "Turbo" => Decimal::from_str("7.0").unwrap(),
        "High" => Decimal::from_str("5.0").unwrap(),
        "Mid" => Decimal::from_str("2.5").unwrap(),
        "Low" => Decimal::from_str("1.0").unwrap(),
        _ => return Err(format!("Unrecognized power level: {}", csv_channel.get("Transmit Power").unwrap()).into()),
    };
    if channel.mode == ChannelMode::FM { // FM specific fields
        channel.fm = Some(FM {
            bandwidth: match csv_channel.get("Band Width").unwrap().as_str() {
                "12.5K" => Decimal::from_str("12.5").unwrap() * Decimal::new(1_000, 0),
                "25K" => Decimal::from_str("25.0").unwrap() * Decimal::new(1_000, 0),
                _ => return Err(format!("Unrecognized bandwidth: {}", csv_channel.get("Band Width").unwrap()).into()),
            },
            squelch_level: 0, // @TODO
            tone_rx: parse_tone(csv_channel.get("CTCSS/DCS Decode").unwrap().as_str()),
            tone_tx: parse_tone(csv_channel.get("CTCSS/DCS Encode").unwrap().as_str()),
        });
    } else if channel.mode == ChannelMode::DMR { // DMR specific fields
        channel.dmr = Some(DMR {
            timeslot: csv_channel.get("Slot").unwrap().parse::<u8>()?,
            color_code: csv_channel.get("Color Code").unwrap().parse::<u8>()?,
            talkgroup: "none".to_string(), // @TODO
        })
    } else {
        return Err("Unparsed channel mode".into());
    }

    Ok(channel)
}

// Convert the CSV zone hashmap into a Zone struct
fn parse_zone_record(csv_zone: &CsvRecord, codeplug: &Codeplug) -> Result<Zone, Box<dyn Error>> {
    let mut zone = Zone {
        name: String::new(),
        channels: Vec::new(),
    };

    zone.name = csv_zone.get("Zone Name").unwrap().to_string();
    // Channels are stored as a list of names, separated by "|"
    let channel_names: Vec<&str> = csv_zone.get("Zone Channel Member").unwrap().split('|').collect();
    for name in channel_names {
        // find the channel by name in the codeplug
        let channel = codeplug.channels.iter().find(|&c| c.name == name);
        match channel {
            Some(c) => zone.channels.push(c.index),
            None => return Err(format!("Channel not found: {}", name).into()),
        }
    }

    Ok(zone)
}

// Convert the CSV DMR ID hashmap into a DMRId struct
fn parse_dmr_id_record(csv_dmr_id: &CsvRecord) -> Result<DmrId, Box<dyn Error>> {
    let dmr_id = DmrId {
        id: csv_dmr_id.get("Radio ID").unwrap().parse::<u32>()?,
        name: csv_dmr_id.get("Name").unwrap().to_string(),
    };

    Ok(dmr_id)
}