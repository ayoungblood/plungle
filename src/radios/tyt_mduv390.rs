// src/radios/tyt_mduv390.rs

use std::error:Error;
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
// - Scan List: zero-index
// - Squelch: [?-?], default 1
// - RX Ref Frequency: default 0
// - TX Ref Frequency: default 0
// - TOT[s]: seconds/15, default 4
// - TOT Rekey Delay[s]: default 0
// - Power: 0,1,2 for low, mid, high
// - Admit Criteria:
// - Auto Scan
// - Rx Only
// - Lone Worker
// - VOX
// - Allow Talkaround
// - Send GPS Info
// - Receive GPS Info
// - Private Call Confirmed
// - Emergency Alarm Ack
// - Data Call Confirmed
// - Allow Interrupt
// - DCDCM Switch
// - Leader/MS
// - Emergency System
// - Contact Name
// - Group List
// - Color Code
// - Repeater Slot
// - In Call Criteria
// - Privacy
// - Privacy No.
// - GPS System
// - CTCSS/DCS Dec
// - CTCSS/DCS Enc
// - RX Signaling System
// - Tx Signaling System
// - QT Reverse
// - Non-QT/DQT Turn-off Freq
// - Display PTT ID
// - Reverse Burst/Turn-off Code
// - Decode 1
// - Decode 2
// - Decode 3
// - Decode 4
// - Decode 5
// - Decode 6
// - Decode 7
// - Decode 8

// contacts.csv
// - Contact Name
// - Call Type
// - Call ID
// - Call Receive Tone

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

fn parse_talkgroup_record(record: &CsvRecord, opt: &Opt) -> Result<DmrTalkgroup, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let talkgroup = DmrTalkgroup {
        id: record.get("Call ID").unwrap().parse::<u32>()?,
        name: record.get("Contact Name").unwrap().to_string(),
        call_type: match record.get("Call Type").unwrap().as_str() {
            "Group" => DmrTalkgroupCallType::Group,
            "Private" => DmrTalkgroupCallType::Private,
            "AllCall" => DmrTalkgroupCallType::AllCall,
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
    static CHANNEL_COUNTER: AtomicUsize = AtomicUsize::new(1);
    channel.index = CHANNEL_COUNTER.fetch_add(1, Ordering::SeqCst);
    channel.name = record.get("Channel Name").unwrap().to_string();
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
            let contact = parse_talkgroup_record(&record, &opt)?;
            // append to codeplug.contacts
            codeplug.contacts.push(contact);
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