// src/radios/anytone_x78.rs

use std::path::Path;
use std::error::Error;
use csv::{Reader, Result as CsvResult, Trim};

use crate::frequency::Frequency;
use crate::power::Power;
use crate::structures::{ChannelMode, Channel, Zone, Talkgroup, TalkgroupList, Codeplug};

// CSV Export Format:
// Channel.CSV
// - No.: Channel Index
// - Channel Name: 16 characters?
// - Receive Frequency: frequency in MHz
// - Transmit Frequency: frequency in MHz
// - Channel Type: [A-Analog, D-Digital]
// - Transmit Power: [Turbo, High, Med, Low], corresponding to ~7W, 5W, 2.5W, 1W
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

pub fn parse(input: &Path) -> Result<Codeplug, Box<dyn Error>> {

    let codeplug = Codeplug {
        channels: Vec::new(),
        zones: Vec::new(),
        lists: Vec::new(),
    };

    // Check for Channel.CSV
    let channel_path = format!("{}/Channel.CSV", input.display());
    if !std::path::Path::new(&channel_path).exists() {
        return Err("Channel.CSV not found".into());
    } else {
        let file = std::fs::File::open(channel_path)?;
        let mut reader = csv::Reader::from_reader(file);
        let headers: Option<Vec<String>> = None;
        for result in reader.records() {
            let record = result?;
            if headers.is_none() {
                return Err("CSV file does not contain headers".into());
            }
            let channel = parse_channel_record(&headers.as_ref().unwrap(), &record)?;
            println!("{:?}", channel);
        }
    }

    // Check for Zone.CSV
    let zone_path = format!("{}/Zone.CSV", input.display());
    if !std::path::Path::new(&zone_path).exists() {
        return Err("Zone.CSV not found".into());
    } else {
        let file = std::fs::File::open(zone_path)?;
        let _reader = csv::Reader::from_reader(file);

    }

    Ok(codeplug)
}

fn parse_channel_record(
    headers: &[String],
    record: &csv::StringRecord,
) -> Result<Channel, Box<dyn Error>> {
    let mut channel = Channel {
        index: 0,
        name: String::new(),
        mode: ChannelMode::AM, // Default mode
        frequency_rx: Frequency::from_mhz(0.0),
        frequency_tx: Frequency::from_mhz(0.0),
        rx_only: false,
        power: Power::from_w(0.0),
        bandwidth: None,
        squelch: None,
        tone_rx: None,
        tone_tx: None,
        timeslot: None,
        color_code: None,
        talkgroup: None,
    };

    for (i, field) in record.iter().enumerate() {
        match headers.get(i) {
            Some(header) => {
                match header.as_str() {
                    "No.:" => channel.index = field.parse::<u32>()?,
                    "Channel Name:" => channel.name = field.to_string(),
                    "Channel Type:" => match field {
                        "A-Analog" => channel.mode = ChannelMode::AM,
                        "D-Digital" => channel.mode = ChannelMode::DMR,
                        _ => return Err(format!("Unknown channel type: {}", field).into()),
                    },
                    "Receive Frequency:" => {
                        channel.frequency_rx = Frequency::from_mhz(field.parse::<f64>()?)
                    }
                    "Transmit Frequency:" => {
                        channel.frequency_tx = Frequency::from_mhz(field.parse::<f64>()?)
                    }
                    "Transmit Power:" => match field {
                        "Turbo" => channel.power = Power::from_mw(500.0),
                        "High" => channel.power = Power::from_mw(500.0),
                        "Med" => channel.power = Power::from_mw(500.0),
                        "Low" => channel.power = Power::from_mw(500.0),
                        _ => return Err(format!("Unknown power level: {}", field).into()),
                    },
                    "Band Width:" => {
                        channel.bandwidth = Some(Frequency::from_khz(field.parse::<f64>()?))
                    }
                    "Squelch Mode:" => channel.squelch = Some(field.to_string()),
                    "CTCSS/DCS Decode:" | "CTCSS/DCS Encode:" => {
                        if field != "Off" {
                            channel.tone_rx = Some(field.to_string());
                        }
                    }
                    "Contact TG/DMR ID:" => {
                        if !field.is_empty() {
                            let id = field.parse::<u32>()?;
                            channel.talkgroup = Some(Talkgroup{id, name: String::new()});
                        }
                    }
                    "Color Code:" => {
                        if !field.is_empty() {
                            channel.color_code = Some(field.parse::<u8>()?);
                        }
                    }
                    "Slot:" => {
                        if !field.is_empty() {
                            channel.timeslot = Some(field.parse::<u8>()?);
                        }
                    }
                    "PTT Prohibit:" => {
                        channel.rx_only = field == "On";
                    }
                    // Handle other fields similarly, parsing values and updating the channel struct
                    _ => {
                        if !header.is_empty() {
                            println!("Unhandled header: {}", header);
                        }
                    }
                }
            }
            None => return Err("CSV record longer than headers".into()),
        }
    }

    Ok(channel)
}