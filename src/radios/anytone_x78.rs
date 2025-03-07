// src/radios/anytone_x78.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
use rust_decimal::prelude::*;
use std::sync::OnceLock;

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
// - Contact: DMR contact (for reasons, this is set to the first digital contact on analog channels)
// - Contact Call Type: [Group Call, All Call, Private Call]
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
// - Dwell Time[s]: default 3.1

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

fn parse_talkgroup_record(record: &CsvRecord, opt: &Opt) -> Result<DmrTalkgroup, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
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

fn parse_talkgroup_list_record(record: &CsvRecord, codeplug: &Codeplug, opt: &Opt) -> Result<DmrTalkgroupList, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
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

// Convert a string into a TxPermit enum
fn parse_tx_permit(tx_permit: &str) -> Option<TxPermit> {
    match tx_permit {
        "Always" => Some(TxPermit::Always),
        "Channel Free" | "ChannelFree" => Some(TxPermit::ChannelFree),
        "Different CDT" => Some(TxPermit::CtcssDcsDifferent),
        "Same Color Code" => Some(TxPermit::ColorCodeSame),
        "Different Color Code" => Some(TxPermit::ColorCodeDifferent),
        _ => None,
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
fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let mut channel = Channel::default();

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
    if channel.frequency_tx >= Decimal::new(174_000_000, 0) { // VHF
        channel.power = match record.get("Transmit Power").unwrap().as_str() {
            "Turbo" => Power::Watts(7.0), // 7W
            "High" => Power::Watts(5.0), // 5W
            "Mid" => Power::Watts(2.5), // 2.5W
            "Low" => Power::Watts(1.0), // 1W
            _ => return Err(format!("Unrecognized power: {}", record.get("Transmit Power").unwrap()).into()),
        };
    } else {
        channel.power = match record.get("Transmit Power").unwrap().as_str() {
            "Turbo" => Power::Watts(7.0), // 6W
            "High" => Power::Watts(5.0), // 5W
            "Mid" => Power::Watts(2.5), // 2.5W
            "Low" => Power::Watts(1.0), // 1W
            _ => return Err(format!("Unrecognized power: {}", record.get("Transmit Power").unwrap()).into()),
        };
    }
    channel.tx_permit = parse_tx_permit(record.get("Busy Lock/TX Permit").unwrap());
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
        // warn if an RX tone is set but squelch mode is not CTCSS/DCS
        if record.get("Squelch Mode").unwrap() != "CTCSS/DCS" && channel.fm.as_ref().unwrap().tone_rx.is_some() {
            uprintln!(opt, Stderr, Color::Yellow, None, "[Warning] {:4} {:24} {}",
                channel.index, channel.name, "RX tone set but squelch mode is not CTCSS/DCS");
            // null out the tone
            channel.fm.as_mut().unwrap().tone_rx = None;
        }
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

    // Check for TalkGroups.CSV, some CPS versions call this ContactTalkGroups.CSV
    let mut talkgroups_path: PathBuf = input_path.clone();
    talkgroups_path.push("TalkGroups.CSV");
    if !talkgroups_path.exists() {
        // try the other name
        talkgroups_path.pop();
        talkgroups_path.push("ContactTalkGroups.CSV");
    }
    // if neither file exist, no problem, we just don't have any talkgroups
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
    // if this file doesn't exist, no problem, we just don't set the radio ID list
    if radio_id_list_path.exists() {
        uprintln!(opt, Stderr, None, 3, "Reading {}", radio_id_list_path.display());
        // Sometimes RadioIDList.CSV has an extra "Name" column in the header, so we need to do some dumb stuff to work around this
        // read the file into a string
        let radio_id_list_content = fs::read_to_string(radio_id_list_path)?;
        // if the first line has two "Name" columns, remove the second one
        let radio_id_list_content = radio_id_list_content.replace("\"Name\",\"Name\"", "\"Name\"");
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(true)
            .from_reader(radio_id_list_content.as_bytes());
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
        uprintln!(opt, Stderr, None, 4, "Writing talkgroup {:width$}: {}", talkgroup.id, talkgroup.name, width = 8);
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
        uprintln!(opt, Stderr, None, 4, "Writing talkgroup list {:width$}: {}", ii + 1, talkgroup_list.name, width = 3);
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

fn write_power(power: &Power) -> String {
    match power {
        Power::Default => "High".to_string(), // 5W
        Power::Watts(w) => {
            if *w >= 7.0 {
                "Turbo".to_string()
            } else if *w >= 5.0 {
                "High".to_string()
            } else if *w >= 2.5 {
                "Mid".to_string()
            } else if *w >= 1.0 {
                "Low".to_string()
            } else {
                "Low".to_string()
            }
        },
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
        Some(TxPermit::ChannelFree) => {
            if channel.mode == ChannelMode::FM {
                "Channel Free".to_string()
            } else {
                "ChannelFree".to_string()
            }
        },
        Some(TxPermit::CtcssDcsDifferent) => "Different CDT".to_string(),
        Some(TxPermit::ColorCodeSame) => "Same Color Code".to_string(),
        Some(TxPermit::ColorCodeDifferent) => "Different Color Code".to_string(),
        None => "Off".to_string(),
    };
    tx_permit
}

fn write_custom_ctcss(channel: &Channel) -> String {
    // if the CTCSS frequency is below 62.5 or above 254.1, write it as a custom frequency
    // @TODO this is an imperfect solution, but it works for now
    // we should be validating against a list of valid CTCSS frequencies
    if let Some(tone) = &channel.fm {
        if let Some(tone_rx) = &tone.tone_rx {
            if tone_rx.mode == ToneMode::CTCSS {
                if let Some(ctcss) = tone_rx.ctcss {
                    if ctcss < Decimal::new(625, 1) || ctcss > Decimal::new(2541, 1) {
                        return format!("{:0.1}", ctcss);
                    }
                }
            }
        }
    }
    "251.1".to_string()
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

fn write_receive_group_list(channel: &Channel, _codeplug: &Codeplug) -> String {
    if let Some(dmr) = &channel.dmr {
        if let Some(talkgroup_list_name) = &dmr.talkgroup_list {
            return talkgroup_list_name.to_string();
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
                "1".to_string(), // Color Code (this has to be set on analog channels or the CPS will refuse to import)
                "1".to_string(), // Slot
                write_scan_list(&channel, &codeplug), // Scan List
                write_receive_group_list(&channel, &codeplug), // Receive Group List
                if channel.rx_only { "On" } else { "Off" }.to_string(), // PTT Prohibit
                "Off".to_string(), // Reverse
                "Off".to_string(), // Simplex TDMA
                "Off".to_string(), // Slot Suit
                "Normal Encryption".to_string(), // AES Digital Encryption
                "Off".to_string(), // Digital Encryption
                "Off".to_string(), // Call Confirmation
                "Off".to_string(), // Talk Around(Simplex)
                "Off".to_string(), // Work Alone
                write_custom_ctcss(&channel), // Custom CTCSS
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
                write_receive_group_list(&channel, &codeplug), // Receive Group List
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
                "1".to_string(), // DMR MODE
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
        uprintln!(opt, Stderr, None, 4, "Writing zone {:width$}: {}", ii + 1, zone.name, width = 3);
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

pub fn write_scanlists(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always) // Anytone CPS expects all fields to be quoted
        .terminator(csv::Terminator::CRLF)
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "No.",
        "Scan List Name",
        "Scan Channel Member",
        "Scan Channel Member RX Frequency",
        "Scan Channel Member TX Frequency",
        "Scan Mode",
        "Priority Channel Select",
        "Priority Channel 1",
        "Priority Channel 1 RX Frequency",
        "Priority Channel 1 TX Frequency",
        "Priority Channel 2",
        "Priority Channel 2 RX Frequency",
        "Priority Channel 2 TX Frequency",
        "Revert Channel",
        "Look Back Time A[s]",
        "Look Back Time B[s]",
        "Dropout Delay Time[s]",
        "Dwell Time[s]",
    ])?;

    for (ii, zone) in codeplug.zones.iter().enumerate() {
        uprintln!(opt, Stderr, None, 4, "Writing scan list {:width$}: {}", ii + 1, zone.name, width = 3);
        let mut channel_names = String::new();
        let mut channel_rx_frequencies = String::new();
        let mut channel_tx_frequencies = String::new();
        // a scan list can only have 50 or fewer channels
        for (jj, name) in zone.channels.iter().take(50).enumerate() {
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

        writer.write_record(&[
            format!("{}", ii + 1), // No.
            zone.name.clone(), // Scan List Name
            channel_names, // Scan Channel Member
            channel_rx_frequencies, // Scan Channel Member RX Frequency
            channel_tx_frequencies, // Scan Channel Member TX Frequency
            "Off".to_string(), // Scan Mode
            "Off".to_string(), // Priority Channel Select
            "Off".to_string(), // Priority Channel 1
            "".to_string(), // Priority Channel 1 RX Frequency
            "".to_string(), // Priority Channel 1 TX Frequency
            "Off".to_string(), // Priority Channel 2
            "".to_string(), // Priority Channel 2 RX Frequency
            "".to_string(), // Priority Channel 2 TX Frequency
            "Selected".to_string(), // Revert Channel
            "2.0".to_string(), // Look Back Time A[s]
            "3.0".to_string(), // Look Back Time B[s]
            "3.1".to_string(), // Dropout Delay Time[s]
            "3.1".to_string(), // Priority Sample Time[s]
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

    // write to TalkGroups.CSV
    let mut talkgroups_path: PathBuf = output_path.clone();
    talkgroups_path.push("TalkGroups.CSV");
    if codeplug.talkgroups.len() > 0 {
        write_talkgroups(codeplug, &talkgroups_path, opt)?;
    }

    // write to ReceiveGroupCallList.CSV
    let mut talkgroup_lists_path: PathBuf = output_path.clone();
    talkgroup_lists_path.push("ReceiveGroupCallList.CSV");
    if codeplug.talkgroups.len() > 0 {
        write_talkgroup_lists(codeplug, &talkgroup_lists_path, opt)?;
    }

    // write to Channel.CSV
    let mut channels_path: PathBuf = output_path.clone();
    channels_path.push("Channel.CSV");
    write_channels(codeplug, &channels_path, opt)?;

    // write to Zone.CSV
    let mut zones_path: PathBuf = output_path.clone();
    zones_path.push("Zone.CSV");
    if codeplug.zones.len() > 0 {
        write_zones(codeplug, &zones_path, opt)?;
    }

    // write to ScanList.CSV
    // Copy zones to scan lists since we don't support scan lists yet @TODO
    let mut scanlists_path: PathBuf = output_path.clone();
    scanlists_path.push("ScanList.CSV");
    if codeplug.zones.len() > 0 {
        write_scanlists(codeplug, &scanlists_path, opt)?;
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
