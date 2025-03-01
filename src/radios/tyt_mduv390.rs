// src/radios/tyt_mduv390.rs

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
//use rust_decimal::prelude::*;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::*;
use crate::structures::*;

static PROPS: OnceLock<structures::RadioProperties> = OnceLock::new();
pub fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.modes = vec![structures::ChannelMode::FM, structures::ChannelMode::DMR];
        props.channels_max = 3000;
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
// TYT MD-UV380 CPS V2.41
/* Files
 * channels.csv
 * contacts.csv
 */

// channels.csv
// - Channel Mode: 1 for analog, 2 for DMR
// - Channel Name: 16 characters max
// - RX Frequency(MHz): unpadded
// - TX Frequency(MHz): unpadded
// - Band Width: 2 for 25kHz, 1 for 20kHz, 0 for 12.5kHz
// - Scan List: 0=None, else one-index
// - Squelch: [0-9], default 1
// - RX Ref Frequency: default 0
// - TX Ref Frequency: default 0
// - TOT[s]: seconds/15, default 4, 0 = infinite
// - TOT Rekey Delay[s]: default 0
// - Power: 0,1,2 for Low, Middle, High
// - Admit Criteria: [0,1,2] for analog (Always, Channel Free, Correct CTCSS/DCS), [0,1,2] for DMR (Always, Channel Free, Color Code)
// - Auto Scan: default 0
// - Rx Only: [0,1] = [off, on], default 0
// - Lone Worker: default 0
// - VOX: default 0
// - Allow Talkaround: [0,1] = [off, on], default 0
// - Send GPS Info: default 0
// - Receive GPS Info: default 0
// - Private Call Confirmed: default 0
// - Emergency Alarm Ack: default 0
// - Data Call Confirmed: default 0
// - Allow Interrupt: default 0
// - DCDCM Switch: default 0
// - Leader/MS: default 1
// - Emergency System: default 0
// - Contact Name: 0 for analog channels, index for DMR channels
// - Group List: 0 for analog channels, index for DMR channels
// - Color Code: [0-15], 1 for analog channels
// - Repeater Slot: [0,1], default 0
// - In Call Criteria: default 0
// - Privacy: default 0
// - Privacy No.: default 0
// - GPS System: default 0
// - CTCSS/DCS Dec: [None,67.0-254.1,D023N,D754I]
// - CTCSS/DCS Enc: [None,67.0-254.1,D023N,D754I]
// - RX Signaling System: default 0
// - Tx Signaling System: default 0
// - QT Reverse: default 0
// - Non-QT/DQT Turn-off Freq: default 2
// - Display PTT ID: default 1
// - Reverse Burst/Turn-off Code: default 1
// - Decode 1 thru Decode 8: default 0

// contacts.csv
// - Contact Name: 16 characters max
// - Call Type: [1,2,3] for [Group, Private, AllCall]
// - Call ID: talkgroup/contact ID
// - Call Receive Tone: default 0, 1 = on

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

fn parse_talkgroup_record(record: &CsvRecord, opt: &Opt) -> Result<DmrTalkgroup, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let talkgroup = DmrTalkgroup {
        id: record.get("Call ID").unwrap().parse::<u32>()?,
        name: record.get("Contact Name").unwrap().to_string(),
        call_type: match record.get("Call Type").unwrap().as_str() {
            "1" => DmrTalkgroupCallType::Group,
            "2" => DmrTalkgroupCallType::Private,
            "3" => DmrTalkgroupCallType::AllCall,
            _ => return Err(format!("Unrecognized call type: {}", record.get("Call Type").unwrap()).into()),
        },
    };
    Ok(talkgroup)
}

fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let mut channel = Channel::default();

    // shared fields
    // there is no index in the CSV, so we have to generate it from a counter
    static CHANNEL_COUNTER: AtomicU32 = AtomicU32::new(1);
    channel.index = CHANNEL_COUNTER.fetch_add(1, Ordering::SeqCst);
    channel.name = record.get("Channel Name").unwrap().to_string();
    channel.mode = match record.get("Channel Mode").unwrap().as_str() {
        "1" => ChannelMode::FM,
        "2" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel mode: {}", record.get("Channel Mode").unwrap()).into()),
    };
    Ok(channel)
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

    // check for contacts.csv, since this is manually exported, give some leeway on the filename
    let mut contacts_path = input_path.clone();
    // search for a file ending with .csv and containing "contact" (case-insensitive) in the name
    let contacts_file = fs::read_dir(&contacts_path)?
        .filter_map(Result::ok)
        .find(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy().to_lowercase();
            file_name.contains("contact") && file_name.ends_with(".csv")
        });
    // if we don't have a contacts.csv file, that's ok
    if let Some(entry) = contacts_file {
        contacts_path.push(entry.file_name());
        uprintln!(opt, Stderr, None, 3, "Reading {}", contacts_path.display());
        let mut reader = csv::Reader::from_path(&contacts_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CsvRecord to Contact struct
            let talkgroup = parse_talkgroup_record(&record, &opt)?;
            // append to codeplug.contacts
            codeplug.talkgroups.push(talkgroup);
        }
    }

    // check for channels.csv, since this is manually exported, give some leeway on the filename
    let mut channels_path = input_path.clone();
    // search for a file ending with .csv and containing "channel" (case-insensitive) in the name
    let channels_file = fs::read_dir(&channels_path)?
        .filter_map(Result::ok)
        .find(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy().to_lowercase();
            file_name.contains("channel") && file_name.ends_with(".csv")
        });

    if let Some(entry) = channels_file {
        channels_path.push(entry.file_name());
    } else {
        uprintln!(opt, Stderr, Color::Red, None, "No channels.csv file found in the directory: {}", input_path.display());
        return Err("Channels file not found".into());
    }
    uprintln!(opt, Stderr, None, 3, "Reading {}", channels_path.display());
    let mut reader = csv::Reader::from_path(&channels_path)?;
    for result in reader.deserialize() {
        let record: CsvRecord = result?;
        // convert from CsvRecord to Channel struct
        let channel = parse_channel_record(&record, &opt)?;
        // append to codeplug.channels
        codeplug.channels.push(channel);
    }

    Ok(codeplug)
}