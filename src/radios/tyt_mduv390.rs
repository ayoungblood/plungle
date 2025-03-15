// src/radios/tyt_mduv390.rs

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
// - Admit Criteria: [0,1,2] for analog (Always, Channel Free, Correct CTCSS/DCS), [0,1,3] for DMR (Always, Channel Free, Color Code)
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
        alert: record.get("Call Receive Tone").unwrap() == "1",
    };
    Ok(talkgroup)
}

fn parse_tone(tone: &str) -> Option<Tone> {
    if tone == "None" {
        return None;
    }
    // if string begins with D, it's a DCS code
    if tone.starts_with("D") {
        return Some(Tone::Dcs(tone.to_string()));
    }
    Some(Tone::Ctcss(tone.parse::<f64>().unwrap()))
}

fn get_talkgroup_by_index(index: u32, codeplug: &Codeplug) -> Option<String> {
    // get the nth talkgroup from the codeplug
    // if index is 0, return None
    if index == 0 {
        return None;
    }
    // if index is greater than the number of talkgroups, return None
    if index > codeplug.talkgroups.len() as u32 {
        return None;
    }
    // return the talkgroup at index - 1 (since the CSV is 1-indexed)
    Some(codeplug.talkgroups[index as usize - 1].name.clone())
}

fn parse_channel_record(record: &CsvRecord, codeplug: &Codeplug, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 4, "    {:?}", record);
    let mut channel = Channel::default();

    // shared fields
    // there is no index in the CSV, so we have to generate it from a counter
    static CHANNEL_INDEX: AtomicUsize = AtomicUsize::new(1);
    channel.index = CHANNEL_INDEX.fetch_add(1, Ordering::SeqCst);
    channel.name = record.get("Channel Name").unwrap().to_string();
    channel.mode = match record.get("Channel Mode").unwrap().as_str() {
        "1" => ChannelMode::FM,
        "2" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel mode: {}", record.get("Channel Mode").unwrap()).into()),
    };
    channel.frequency_rx = Decimal::from_str(record.get("RX Frequency(MHz)").unwrap().trim())? * Decimal::new(1_000_000, 0);
    channel.frequency_tx = Decimal::from_str(record.get("TX Frequency(MHz)").unwrap().trim())? * Decimal::new(1_000_000, 0);
    channel.rx_only = record.get("Rx Only").unwrap().as_str() == "1";
    if record.get("TOT[s]").unwrap() == "0" {
        channel.tx_tot = Timeout::Infinite;
    } else {
        channel.tx_tot = Timeout::Seconds(record.get("TOT[s]").unwrap().parse::<u32>()? * 15);
    }
    channel.power = match record.get("Power").unwrap().as_str() {
        "0" => Power::Watts(1.0), // Low
        "1" => Power::Watts(2.5), // Middle
        "2" => Power::Watts(5.0), // High
        _ => return Err(format!("Unrecognized power level: {}", record.get("Power").unwrap()).into()),
    };
    channel.tx_permit = match record.get("Admit Criteria").unwrap().as_str() {
        "0" => Some(TxPermit::Always),
        "1" => Some(TxPermit::ChannelFree),
        "2" => Some(TxPermit::CtcssDcsDifferent),
        "3" => Some(TxPermit::ColorCodeSame),
        _ => return Err(format!("Unrecognized admit criteria: {}", record.get("Admit Criteria").unwrap()).into()),
    };
    // mode specific fields
    match channel.mode {
        ChannelMode::FM => {
            channel.fm = Some(FmChannel {
                bandwidth: match record.get("Band Width").unwrap().as_str() {
                    "2" => Decimal::new(25_000, 0), // 25kHz
                    "1" => Decimal::new(20_000, 0), // 20kHz
                    "0" => Decimal::new(12_500, 0), // 12.5kHz
                    _ => return Err(format!("Unrecognized bandwidth: {}", record.get("Band Width").unwrap()).into()),
                },
                squelch: Squelch::Percent(record.get("Squelch").unwrap().parse::<u8>()? * 10),
                tone_rx: parse_tone(record.get("CTCSS/DCS Dec").unwrap().trim()),
                tone_tx: parse_tone(record.get("CTCSS/DCS Enc").unwrap().trim()),
            });
        }
        ChannelMode::DMR => {
            channel.dmr = Some(DmrChannel {
                timeslot: record.get("Repeater Slot").unwrap().parse::<u8>()? + 1,
                color_code: record.get("Color Code").unwrap().parse::<u8>()?,
                talkgroup: get_talkgroup_by_index(
                    record.get("Contact Name").unwrap().parse::<u32>()?,
                    &codeplug,
                ),
                talkgroup_list: None, // CPS does not export talkgroup lists
                id_name: None,
            });
        }
        _ => {}
    }

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
        let channel = parse_channel_record(&record, &codeplug, &opt)?;
        // append to codeplug.channels
        codeplug.channels.push(channel);
    }

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

fn write_talkgroups(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    // write contacts.csv
    let mut writer = csv::WriterBuilder::new()
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "Contact Name",
        "Call Type",
        "Call ID",
        "Call Receive Tone",
    ])?;

    for talkgroup in &codeplug.talkgroups {
        uprintln!(opt, Stderr, None, 4, "Writing talkgroup: {}", talkgroup.name);
        writer.write_record(&[
            talkgroup.name.clone(), // Contact Name
            match talkgroup.call_type {
                DmrTalkgroupCallType::Group => "1".to_string(),
                DmrTalkgroupCallType::Private => "2".to_string(),
                DmrTalkgroupCallType::AllCall => "3".to_string(),
            },
            talkgroup.id.to_string(), // Call ID
            if talkgroup.alert { "1".to_string() } else { "0".to_string() }, // Call Receive Tone
        ])?;
    }
    writer.flush()?;
    Ok(())
}

fn write_squelch(squelch: &Squelch) -> String {
    match squelch {
        Squelch::Default => "1".to_string(),
        Squelch::Percent(p) => (p / 10).to_string(),
    }
}

fn write_tx_tot(tx_tot: &Timeout) -> String {
    match tx_tot {
        Timeout::Default => "4".to_string(),
        Timeout::Seconds(s) => (s / 15).to_string(),
        Timeout::Infinite => "0".to_string(),
    }
}

fn write_tx_permit(tx_permit: &Option<TxPermit>) -> String {
    match tx_permit {
        Some(p) => match p {
            TxPermit::Always => "0".to_string(),
            TxPermit::ChannelFree => "1".to_string(),
            TxPermit::CtcssDcsDifferent => "2".to_string(),
            TxPermit::ColorCodeSame => "3".to_string(),
            _ => "0".to_string(),
        },
        None => "0".to_string(),
    }
}

fn write_tone(tone: &Option<Tone>) -> String {
    match tone {
        Some(Tone::Ctcss(value)) => format!("{:.1}", value),
        Some(Tone::Dcs(code)) => code.clone(),
        None => "None".to_string(),
    }
}

fn write_contact(channel: &Channel, codeplug: &Codeplug) -> String {
    match &channel.dmr.as_ref().unwrap().talkgroup {
        Some(tg) => {
            // find the talkgroup in the codeplug
            let index = codeplug.talkgroups.iter().position(|x| x.name == *tg).unwrap();
            (index + 1).to_string()
        },
        None => "0".to_string(),
    }
}

fn write_group_list(_channel: &Channel, _codeplug: &Codeplug) -> String {
    "0".to_string()
}

fn write_power(power: &Power) -> String {
    match power {
        Power::Default => "2".to_string(), // Default to High
        Power::Watts(w) if *w >= 5.0 => "2".to_string(), // High
        Power::Watts(w) if *w >= 2.5 => "1".to_string(), // Middle
        Power::Watts(w) if *w >= 1.0 => "0".to_string(), // Low
        _ => "0".to_string(),
    }
}

fn write_channels(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    uprintln!(opt, Stderr, None, 2, "{}:{}()", file!(), function!());
    uprintln!(opt, Stderr, None, 4, "props = {:?}", get_props());

    // write channels.csv
    let mut writer = csv::WriterBuilder::new()
        .from_path(path)?;

    // write the header
    writer.write_record(&[
        "Channel Mode",
        "Channel Name",
        "RX Frequency(MHz)",
        "TX Frequency(MHz)",
        "Band Width",
        "Scan List",
        "Squelch",
        "RX Ref Frequency",
        "TX Ref Frequency",
        "TOT[s]",
        "TOT Rekey Delay[s]",
        "Power",
        "Admit Criteria",
        "Auto Scan",
        "Rx Only",
        "Lone Worker",
        "VOX",
        "Allow Talkaround",
        "Send GPS Info",
        "Receive GPS Info",
        "Private Call Confirmed",
        "Emergency Alarm Ack",
        "Data Call Confirmed",
        "Allow Interrupt",
        "DCDM Switch",
        "Leader/MS",
        "Emergency System",
        "Contact Name",
        "Group List",
        "Color Code",
        "Repeater Slot",
        "In Call Criteria",
        "Privacy",
        "Privacy No.",
        "GPS System",
        "CTCSS/DCS Dec",
        "CTCSS/DCS Enc",
        "Rx Signaling System",
        "Tx Signaling System",
        "QT Reverse",
        "Non-QT/DQT Turn-off Freq",
        "Display PTT ID",
        "Reverse Burst/Turn-off Code",
        "Decode 1",
        "Decode 2",
        "Decode 3",
        "Decode 4",
        "Decode 5",
        "Decode 6",
        "Decode 7",
        "Decode 8",
    ])?;

    for channel in &codeplug.channels {
        uprintln!(opt, Stderr, None, 4, "Writing channel {:width$}: {}", channel.index, channel.name, width = get_props().channel_index_width);
        if channel.mode == ChannelMode::FM {
            writer.write_record(&[
                "1".to_string(), // Channel Mode
                channel.name.clone(), // Channel Name
                format!("{:.5}", channel.frequency_rx / Decimal::new(1_000_000, 0)), // RX Frequency(MHz)
                format!("{:.5}", channel.frequency_tx / Decimal::new(1_000_000, 0)), // TX Frequency(MHz)
                match channel.fm.as_ref().unwrap().bandwidth {
                    bw if bw == Decimal::new(25_000, 0) => "2".to_string(), // 25kHz
                    bw if bw == Decimal::new(20_000, 0) => "1".to_string(), // 20kHz
                    bw if bw == Decimal::new(12_500, 0) => "0".to_string(), // 12.5kHz
                    _ => return Err("Unrecognized bandwidth".into()),
                },
                "0".to_string(), // Scan List
                write_squelch(&channel.fm.as_ref().unwrap().squelch), // Squelch
                "0".to_string(), // RX Ref Frequency
                "0".to_string(), // TX Ref Frequency
                write_tx_tot(&channel.tx_tot), // TOT[s]
                "0".to_string(), // TOT Rekey Delay[s]
                write_power(&channel.power), // Power
                write_tx_permit(&channel.tx_permit), // Admit Criteria (Always)
                "0".to_string(), // Auto Scan
                if channel.rx_only { "1" } else { "0" }.to_string(), // Rx Only
                "0".to_string(), // Lone Worker
                "0".to_string(), // VOX
                "0".to_string(), // Allow Talkaround
                "0".to_string(), // Send GPS Info
                "0".to_string(), // Receive GPS Info
                "0".to_string(), // Private Call Confirmed
                "0".to_string(), // Emergency Alarm Ack
                "0".to_string(), // Data Call Confirmed
                "0".to_string(), // Allow Interrupt
                "0".to_string(), // DCDCM Switch
                "1".to_string(), // Leader/MS
                "0".to_string(), // Emergency System
                "0".to_string(), // Contact Name
                "0".to_string(), // Group List
                "1".to_string(), // Color Code
                "0".to_string(), // Repeater Slot
                "0".to_string(), // In Call Criteria
                "0".to_string(), // Privacy
                "0".to_string(), // Privacy No.
                "0".to_string(), // GPS System
                write_tone(&channel.fm.as_ref().unwrap().tone_rx), // CTCSS/DCS Dec
                write_tone(&channel.fm.as_ref().unwrap().tone_tx), // CTCSS/DCS Enc
                "0".to_string(), // RX Signaling System
                "0".to_string(), // Tx Signaling System
                "0".to_string(), // QT Reverse
                "2".to_string(), // Non-QT/DQT Turn-off Freq
                "1".to_string(), // Display PTT ID
                "1".to_string(), // Reverse Burst/Turn-off Code
                "0".to_string(), // Decode 1
                "0".to_string(), // Decode 2
                "0".to_string(), // Decode 3
                "0".to_string(), // Decode 4
                "0".to_string(), // Decode 5
                "0".to_string(), // Decode 6
                "0".to_string(), // Decode 7
                "0".to_string(), // Decode 8
            ])?;
        } else if channel.mode == ChannelMode::DMR {
            writer.write_record(&[
                "2".to_string(), // Channel Mode
                channel.name.clone(), // Channel Name
                format!("{:.5}", channel.frequency_rx / Decimal::new(1_000_000, 0)), // RX Frequency(MHz)
                format!("{:.5}", channel.frequency_tx / Decimal::new(1_000_000, 0)), // TX Frequency(MHz)
                "0".to_string(), // Band Width
                "0".to_string(), // Scan List
                "1".to_string(), // Squelch
                "0".to_string(), // RX Ref Frequency
                "0".to_string(), // TX Ref Frequency
                write_tx_tot(&channel.tx_tot), // TOT[s]
                "0".to_string(), // TOT Rekey Delay[s]
                write_power(&channel.power), // Power,
                write_tx_permit(&channel.tx_permit), // Admit Criteria (Always)
                "0".to_string(), // Auto Scan
                if channel.rx_only { "1" } else { "0" }.to_string(), // Rx Only
                "0".to_string(), // Lone Worker
                "0".to_string(), // VOX
                "0".to_string(), // Allow Talkaround
                "0".to_string(), // Send GPS Info
                "0".to_string(), // Receive GPS Info
                "0".to_string(), // Private Call Confirmed
                "0".to_string(), // Emergency Alarm Ack
                "0".to_string(), // Data Call Confirmed
                "0".to_string(), // Allow Interrupt
                "0".to_string(), // DCDCM Switch
                "1".to_string(), // Leader/MS
                "0".to_string(), // Emergency System
                write_contact(&channel, &codeplug), // Contact Name
                write_group_list(&channel, &codeplug), // Group List
                channel.dmr.as_ref().unwrap().color_code.to_string(), // Color Code
                (channel.dmr.as_ref().unwrap().timeslot - 1).to_string(), // Repeater Slot
                "0".to_string(), // In Call Criteria
                "0".to_string(), // Privacy
                "0".to_string(), // Privacy No.
                "0".to_string(), // GPS System
                "None".to_string(), // CTCSS/DCS Dec
                "None".to_string(), // CTCSS/DCS Enc
                "0".to_string(), // RX Signaling System
                "0".to_string(), // Tx Signaling System
                "0".to_string(), // QT Reverse
                "2".to_string(), // Non-QT/DQT Turn-off Freq
                "1".to_string(), // Display PTT ID
                "1".to_string(), // Reverse Burst/Turn-off Code
                "0".to_string(), // Decode 1
                "0".to_string(), // Decode 2
                "0".to_string(), // Decode 3
                "0".to_string(), // Decode 4
                "0".to_string(), // Decode 5
                "0".to_string(), // Decode 6
                "0".to_string(), // Decode 7
                "0".to_string(), // Decode 8
            ])?;
        }
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

    // write contacts.csv
    let mut contacts_path = output_path.clone();
    contacts_path.push("contacts.csv");
    if codeplug.talkgroups.len() > 0 {
        write_talkgroups(&codeplug, &contacts_path, opt)?;
    }

    // write channels.csv
    let mut channels_path = output_path.clone();
    channels_path.push("channels.csv");
    write_channels(&codeplug, &channels_path, opt)?;

    Ok(())
}
