// src/radios/anytone_x78.rs
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
        props.channels_max = 4000;
        props.channel_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props
    })
}

// CSV Export Format
// Anytone D878UV CPS Version 3.04
/* Files
 * 2ToneEncode.CSV
 * 5ToneEncode.CSV
 * AESEncryptionCode.CSV
 * AlertTone.CSV
 * AnalogAddressBook.CSV
 * APRS.CSV
 * ARC4EncryptionCode.CSV
 * AutoRepeaterOffsetFrequencys.CSV
 * Channel.CSV
 * DigitalContactList.CSV
 * DTMFEncode.CSV
 * FM.CSV
 * GPSRoaming.CSV
 * HotKey_HotKey.CSV
 * HotKey_QuickCall.CSV
 * HotKey_State.CSV
 * OptionalSetting.CSV
 * PrefabricatedSMS.CSV
 * RadioIDList.CSV
 * ReceiveGroupCallList.CSV
 * RoamingChannel.CSV
 * RoamingZone.CSV
 * ScanList.CSV
 * TalkGroups.CSV
 * Zone.CSV
 */

// Channel.CSV
// - No.: channel Index
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
// - Busy Lock/TX Permit: [Off, Always, Different CDT, Channel Free, Same Color Code, Different Color Code]
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
// - Digital Encryption: [Off, ???]
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

// RadioIDList.CSV
// - No.: radio ID index
// - Radio ID: radio ID
// - Name: radio ID name

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

// TalkGroups.CSV
// - No.: DMR talkgroup index
// - Radio ID: DMR talkgroup ID
// - Name: DMR talkgroup name (@TODO length??)
// - Call Type: [Group Call, All Call, Private Call]
// - Call Alert: [None, ???]

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

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

fn parse_talkgroup_record(record: &CsvRecord) -> Result<DmrTalkgroup, Box<dyn Error>> {
    let talkgroup = DmrTalkgroup {
        id: record.get("Radio ID").unwrap().parse::<u32>()?,
        name: record.get("Name").unwrap().to_string(),
        call_type: match record.get("Call Type").unwrap().as_str() {
            "Group Call" => DmrTalkgroupCallType::Group,
            "Private Call" => DmrTalkgroupCallType::Private,
            "All Call" => DmrTalkgroupCallType::AllCall,
            _ => return Err(format!("Unrecognized call type: {}", record.get("Call Type").unwrap()).into()),
        },
    };

    Ok(talkgroup)
}

fn parse_talkgroup_list_record(record: &CsvRecord, codeplug: &Codeplug) -> Result<DmrTalkgroupList, Box<dyn Error>> {
    let mut talkgroup_list = DmrTalkgroupList {
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
fn parse_channel_record(record: &CsvRecord) -> Result<Channel, Box<dyn Error>> {
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

    channel.index = record.get("No.").unwrap().parse::<u32>()?;
    channel.name = record.get("Channel Name").unwrap().to_string();
    channel.mode = match record.get("Channel Type").unwrap().as_str() {
        "A-Analog" => ChannelMode::FM,
        "D-Digital" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel type: {}", record.get("Channel Type").unwrap()).into()),
    };
    channel.frequency_rx = Decimal::from_str(record.get("Receive Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.frequency_tx = Decimal::from_str(record.get("Transmit Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.rx_only = record.get("PTT Prohibit").unwrap() == "On";
    channel.power = match record.get("Transmit Power").unwrap().as_str() {
        "Turbo" => Decimal::from_str("7.0").unwrap(),
        "High" => Decimal::from_str("5.0").unwrap(),
        "Mid" => Decimal::from_str("2.5").unwrap(),
        "Low" => Decimal::from_str("1.0").unwrap(),
        _ => return Err(format!("Unrecognized power level: {}", record.get("Transmit Power").unwrap()).into()),
    };
    if channel.mode == ChannelMode::FM { // FM specific fields
        channel.fm = Some(FmChannel {
            bandwidth: match record.get("Band Width").unwrap().as_str() {
                "12.5K" => Decimal::from_str("12.5").unwrap() * Decimal::new(1_000, 0),
                "25K" => Decimal::from_str("25.0").unwrap() * Decimal::new(1_000, 0),
                _ => return Err(format!("Unrecognized bandwidth: {}", record.get("Band Width").unwrap()).into()),
            },
            squelch_level: 0, // @TODO
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
            Some(c) => zone.channels.push(c.name.clone()),
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

    // Check for TalkGroups.CSV
    let mut talkgroups_path: PathBuf = input_path.clone();
    talkgroups_path.push("TalkGroups.CSV");
    // if TalkGroups.CSV doesn't exist, no problem, we just don't have any talkgroups
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

    // Check for ReceiveGroupCallList.CSV
    let mut talkgroup_lists_path: PathBuf = input_path.clone();
    talkgroup_lists_path.push("ReceiveGroupCallList.CSV");
    // if this file doesn't exist, no problem, we just don't have any talkgroup lists
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

    // Check for Channel.CSV
    let mut channels_path: PathBuf = input_path.clone();
    channels_path.push("Channel.CSV");
    if !channels_path.exists() {
        return Err("Channel.CSV not found".into());
    } else {
        dprintln!(opt.verbose, 3, "Reading {}", channels_path.display());
        let mut reader = csv::Reader::from_path(channels_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Channel struct
            let channel = parse_channel_record(&record)?;
            // Anytone D878UV stores VFO A/B at 4001/4002, skip these
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

// WRITE //////////////////////////////////////////////////////////////////////

pub fn write_talkgroups(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    dprintln!(opt.verbose, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always) // Anytone CPS expects all fields to be quoted
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
        dprintln!(opt.verbose, 4, "Writing talkgroup {:width$}: {}", talkgroup.id, talkgroup.name, width = 8);
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
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    dprintln!(opt.verbose, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always) // Anytone CPS expects all fields to be quoted
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
        dprintln!(opt.verbose, 4, "Writing talkgroup list {:width$}: {}", ii + 1, talkgroup_list.name, width = 3);
        let mut contact = String::new();
        let mut contact_id = String::new();
        for (jj, talkgroup) in talkgroup_list.talkgroups.iter().enumerate() {
            if jj > 0 {
                contact.push_str("|");
                contact_id.push_str("|");
            }
            contact.push_str(&talkgroup.name);
            contact_id.push_str(&talkgroup.id.to_string());
        }
        writer.write_record(&[
            format!("{}", ii + 1), // No.
            talkgroup_list.name.clone(), // Group Name
            contact, // Contact
            contact_id, // Contact TG/DMR ID
        ])?;
    }

    writer.flush()?;

    Ok(())
}

pub fn write_power(channel: &Channel) -> String {
    if channel.power >= Decimal::from_str("7.0").unwrap() {
        return "Turbo".to_string();
    } else if channel.power >= Decimal::from_str("5.0").unwrap() {
        return "High".to_string();
    } else if channel.power >= Decimal::from_str("2.5").unwrap() {
        return "Mid".to_string();
    } else if channel.power >= Decimal::from_str("1.0").unwrap() {
        return "Low".to_string();
    } else {
        return "Low".to_string();
    }
}

pub fn write_channels(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    dprintln!(opt.verbose, 1, "Writing {}", path.display());

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
        "PTT Prohibit",
        "Reverse",
        "Simplex TDMA",
        "Slot Suit",
        "AES Digital Encryption",
        "Digital Encryption",
        "Call Confirmation",
        "Talk Around(Simplex)",
        "Work Alone",
        "Custom CTCSS",
        "2TONE Decode",
        "Ranging",
        "Through Mode",
        "APRS RX",
        "Analog APRS PTT Mode",
        "Digital APRS PTT Mode",
        "APRS Report Type",
        "Digital APRS Report Channel",
        "Correct Frequency[Hz]",
        "SMS Confirmation",
        "Exclude channel from roaming",
        "DMR MODE",
        "DataACK Disable",
        "R5toneBot",
        "R5ToneEot",
        "Auto Scan",
        "Ana Aprs Mute",
        "Send Talker Alias",
        "AnaAprsTxPath",
        "ARC4",
        "ex_emg_kind",
    ])?;

    for channel in &codeplug.channels {
        dprintln!(opt.verbose, 4, "Writing channel {:width$}: {}", channel.index, channel.name, width = get_props().channel_index_width);
        if channel.mode == ChannelMode::FM {
            writer.write_record(&[
                channel.index.to_string(), // No.
                channel.name.clone(), // Channel Name
                format!("{:0.5}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Receive Frequency
                format!("{:0.5}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Transmit Frequency
                "A-Analog".to_string(), // Channel Type
                write_power(channel), // Transmit Power
                format!("{}K", (channel.fm.as_ref().unwrap().bandwidth / Decimal::new(1_000, 0)).to_f64().unwrap()), // Band Width
                if let Some(tone) = &channel.fm.as_ref().unwrap().tone_rx {
                    match tone.mode {
                        ToneMode::CTCSS => format!("{:0.1}", tone.ctcss.as_ref().unwrap()),
                        ToneMode::DCS => tone.dcs.as_ref().unwrap().to_string(),
                    }
                } else {
                    "Off".to_string()
                }, // CTCSS/DCS Decode
                if let Some(tone) = &channel.fm.as_ref().unwrap().tone_tx {
                    match tone.mode {
                        ToneMode::CTCSS => format!("{:0.1}", tone.ctcss.as_ref().unwrap()),
                        ToneMode::DCS => tone.dcs.as_ref().unwrap().to_string(),
                    }
                } else {
                    "Off".to_string()
                }, // CTCSS/DCS Encode
                "".to_string(), // Contact
                "".to_string(), // Contact Call Type
                "".to_string(), // Contact TG/DMR ID
                "".to_string(), // Radio ID
                "".to_string(), // Busy Lock/TX Permit
                if channel.fm.as_ref().unwrap().tone_rx.is_some() {
                    "CTCSS/DCS".to_string()
                } else {
                    "Carrier".to_string()
                }, // Squelch Mode
                "".to_string(), // Optional Signal
                "1".to_string(), // DTMF ID
                "1".to_string(), // 2Tone ID
                "1".to_string(), // 5Tone ID
                "Off".to_string(), // PTT ID
                "".to_string(), // Color Code
                "".to_string(), // Slot
                "".to_string(), // Scan List
                "".to_string(), // Receive Group List
                if channel.rx_only { "On" } else { "Off" }.to_string(), // PTT Prohibit
                "Off".to_string(), // Reverse
                "Off".to_string(), // Simplex TDMA
                "Off".to_string(), // Slot Suit
                "Normal Encryption".to_string(), // AES Digital Encryption
                "Off".to_string(), // Digital Encryption
                "Off".to_string(), // Call Confirmation
                "Off".to_string(), // Talk Around(Simplex)
                "Off".to_string(), // Work Alone
                "251.1".to_string(), // Custom CTCSS
                "0".to_string(), // 2TONE Decode
                "Off".to_string(), // Ranging
                "Off".to_string(), // Through Mode
                "Off".to_string(), // APRS RX
                "Off".to_string(), // Analog APRS PTT Mode
                "Off".to_string(), // Digital APRS PTT Mode
                "Off".to_string(), // APRS Report Type
                "1".to_string(), // Digital APRS Report Channel
                "0".to_string(), // Correct Frequency[Hz]
                "Off".to_string(), // SMS Confirmation
                "0".to_string(), // Exclude channel from roaming
                "0".to_string(), // DMR MODE
                "0".to_string(), // DataACK Disable
                "0".to_string(), // R5toneBot
                "0".to_string(), // R5ToneEot
                "0".to_string(), // Auto Scan
                "0".to_string(), // Ana Aprs Mute
                "0".to_string(), // Send Talker Alias
                "0".to_string(), // AnaAprsTxPath
                "0".to_string(), // ARC4
                "0".to_string(), // ex_emg_kind
            ])?;
        } else if channel.mode == ChannelMode::DMR {
            writer.write_record(&[
                channel.index.to_string(), // No.
                channel.name.clone(), // Channel Name
                format!("{:0.5}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Receive Frequency
                format!("{:0.5}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // Transmit Frequency
                "D-Digital".to_string(), // Channel Type
                write_power(channel), // Transmit Power
                "".to_string(), // Band Width
                "".to_string(), // CTCSS/DCS Decode
                "".to_string(), // CTCSS/DCS Encode
                channel.dmr.as_ref().unwrap().talkgroup.as_ref().unwrap().to_string(), // Contact
                "Group Call".to_string(), // Contact Call Type
                "".to_string(), // Contact TG/DMR ID
                "".to_string(), // Radio ID
                "".to_string(), // Busy Lock/TX Permit
                "".to_string(), // Squelch Mode
                "".to_string(), // Optional Signal
                "1".to_string(), // DTMF ID
                "1".to_string(), // 2Tone ID
                "1".to_string(), // 5Tone ID
                "Off".to_string(), // PTT ID
                channel.dmr.as_ref().unwrap().color_code.to_string(), // Color Code
                channel.dmr.as_ref().unwrap().timeslot.to_string(), // Slot
                "".to_string(), // Scan List
                "".to_string(), // Receive Group List
                if channel.rx_only { "On" } else { "Off" }.to_string(), // PTT Prohibit
                "Off".to_string(), // Reverse
                "Off".to_string(), // Simplex TDMA
                "Off".to_string(), // Slot Suit
                "Normal Encryption".to_string(), // AES Digital Encryption
                "Off".to_string(), // Digital Encryption
                "Off".to_string(), // Call Confirmation
                "Off".to_string(), // Talk Around(Simplex)
                "Off".to_string(), // Work Alone
                "251.1".to_string(), // Custom CTCSS
                "0".to_string(), // 2TONE Decode
                "Off".to_string(), // Ranging
                "Off".to_string(), // Through Mode
                "Off".to_string(), // APRS RX
                "Off".to_string(), // Analog APRS PTT Mode
                "Off".to_string(), // Digital APRS PTT Mode
                "Off".to_string(), // APRS Report Type
                "1".to_string(), // Digital APRS Report Channel
                "0".to_string(), // Correct Frequency[Hz]
                "Off".to_string(), // SMS Confirmation
                "0".to_string(), // Exclude channel from roaming
                "0".to_string(), // DMR MODE
                "0".to_string(), // DataACK Disable
                "0".to_string(), // R5toneBot
                "0".to_string(), // R5ToneEot
                "0".to_string(), // Auto Scan
                "0".to_string(), // Ana Aprs Mute
                "0".to_string(), // Send Talker Alias
                "0".to_string(), // AnaAprsTxPath
                "0".to_string(), // ARC4
                "0".to_string(), // ex_emg_kind
            ])?;
        } else {
            cprintln!(ANSI_C_YLW, "Unsupported channel mode: index = {}, mode = {:?}", channel.index, channel.mode);
        }
    }

    writer.flush()?;

    Ok(())
}

pub fn write_zones(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    dprintln!(opt.verbose, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always) // Anytone CPS expects all fields to be quoted
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Zone Name",
        "Zone Channel Member",
        "Zone Channel Member RX Frequency",
        "Zone Channel Member TX Frequency",
        "A Channel",
        "A Channel RX Frequency",
        "A Channel TX Frequency",
        "B Channel",
        "B Channel RX Frequency",
        "B Channel TX Frequency",
        "Zone Hide ",
    ])?;

    for (ii, zone) in codeplug.zones.iter().enumerate() {
        dprintln!(opt.verbose, 4, "Writing zone {:width$}: {}", ii + 1, zone.name, width = 3);
        let mut channel_names = String::new();
        let mut channel_rx_frequencies = String::new();
        let mut channel_tx_frequencies = String::new();
        for (jj, name) in zone.channels.iter().enumerate() {
            if jj > 0 {
                channel_names.push_str("|");
                channel_rx_frequencies.push_str("|");
                channel_tx_frequencies.push_str("|");
            }
            let channel = codeplug.channels.iter().find(|&c| c.name == *name).unwrap();
            channel_names.push_str(&channel.name);
            channel_rx_frequencies.push_str(&format!("{:0.5}", (channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()));
            channel_tx_frequencies.push_str(&format!("{:0.5}", (channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()));
        }
        // get the first channel in the zone
        let first_channel = codeplug.channels.iter().find(|&c| c.name == zone.channels[0]).unwrap();
        // get the second channel in the zone, or the first channel if there is only one
        let second_channel = codeplug.channels.iter().find(|&c| c.name == *zone.channels.get(1).unwrap_or(&zone.channels[0])).unwrap();
        writer.write_record(&[
            format!("{}", ii + 1), // No.
            zone.name.clone(), // Zone Name
            channel_names, // Zone Channel Member
            channel_rx_frequencies, // Zone Channel Member RX Frequency
            channel_tx_frequencies, // Zone Channel Member TX Frequency
            first_channel.name.clone(), // A Channel
            format!("{:0.5}", (first_channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // A Channel RX Frequency
            format!("{:0.5}", (first_channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // A Channel TX Frequency
            second_channel.name.clone(), // B Channel
            format!("{:0.5}", (second_channel.frequency_rx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // B Channel RX Frequency
            format!("{:0.5}", (second_channel.frequency_tx / Decimal::new(1_000_000, 0)).to_f64().unwrap()), // B Channel TX Frequency
            "0".to_string(), // Zone Hide
        ])?;
    }

    writer.flush()?;
    Ok(())
}

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

    // write to TalkGroups.CSV
    let mut talkgroups_path: PathBuf = opt.output.clone().unwrap();
    talkgroups_path.push(if opt.excel { "TalkGroups2.CSV" } else { "TalkGroups.CSV" });
    if codeplug.talkgroups.len() > 0 {
        write_talkgroups(codeplug, &talkgroups_path, opt)?;
    }

    // write to ReceiveGroupCallList.CSV
    let mut talkgroup_lists_path: PathBuf = opt.output.clone().unwrap();
    talkgroup_lists_path.push(if opt.excel { "ReceiveGroupCallList2.CSV" } else { "ReceiveGroupCallList.CSV" });
    if codeplug.talkgroups.len() > 0 {
        write_talkgroup_lists(codeplug, &talkgroup_lists_path, opt)?;
    }

    // write to Channel.CSV
    let mut channels_path: PathBuf = opt.output.clone().unwrap();
    channels_path.push(if opt.excel { "Channel2.CSV" } else { "Channel.CSV" });
    write_channels(codeplug, &channels_path, opt)?;

    // write to Zone.CSV
    let mut zones_path: PathBuf = opt.output.clone().unwrap();
    zones_path.push(if opt.excel { "Zone2.CSV" } else { "Zone.CSV" });
    if codeplug.zones.len() > 0 {
        write_zones(codeplug, &zones_path, opt)?;
    }

    Ok(())
}
