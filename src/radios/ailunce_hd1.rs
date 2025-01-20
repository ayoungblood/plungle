// src/radios/ailunce-hd1.rs

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
fn get_props() -> &'static structures::RadioProperties {
    PROPS.get_or_init(|| {
        let mut props = structures::RadioProperties::default();
        props.channels_max = 3000;
        props.channel_name_width_max = 14;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props
    })
}

// HD1(GPS) CPS v3.03
// HD1 CPS is pretty cursed and only supports export/import of the following:
// * Channels
// * Contacts
// * Priority Contacts
// This implies that most codeplugs will take quite a lot of manual fixing-up

// Channels.csv
// - No.: Channel index, rows 1-2 are VFO-A, VFO-B
// - Channel Type: [Analog CH,Digital CH]
// - Channel Alias: channel name
// - Rx Frequency: receive frequency
// - Tx Frequency: transmit frequency
// - Tx Power: [Low,??,High]
// - TOT: timeout timer, [75S,180S,???]
// - VOX: [No,??]
// - VOX Level: [1,2,??]\
// - Scan Add/Step: scan add for non-VFO, step for VFO, [Yes,No] for regular channels
// - Channel Work Alone: [No,??]
// - Default to Talkaround: [No,??]
// - Band Width: [12.5K,25K]
// - Dec QT/DQT: [None,62.5..254.1,D023N..D754I]
// - Enc QT/DQT: [None,62.5..254.1,D023N..D754I]
// - Tx Authority: [Allow TX,Channel Free,Prohibit TX]
// - Relay: [No,??]
// - Work Mode: Simplex for analog channels, Repeat for digital channels
// - Slot: [Slot1,Slot2], Slot1 for analog
// - ID Setting: Radio ID name
// - Color Code: [0..15], 1 for analog channels
// - Encryption: [No,??]
// - Encryption Type: [Normal Mode,??]
// - Encryption Key: [1,??]
// - Promiscuous: [No,??]
// - Tx Authority: [Always,Color Code,Channel Free] note that this is a duplicate column!
// - Kill Code: [None,??]
// - WakeUp Code: [None,??]
// - Contacts: [Priority Contacts: ALLCALL,Priority Contacts: BM Global] ??
// - Rx Group Lists: [Custom]
// - Group Lists 1
// - ...
// - Group Lists 33
// - GPS: [No,??]
// - Send GPS Info: [No,??]
// - Receive GPS Info: [No,??]
// - GPS Timing Report: [OFF,??]
// - GPS Timing Report TX Contact: [TX Contact,??]

// Contacts.csv
// - Call Type:
// - Contacts Alias:
// - City:
// - Province:
// - Country:
// - Call ID:

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

// Convert a CTCSS/DCS string into a Tone struct
// OpenGD77 stores CTCSS/DCS as follows:
// - "None" for no tone
// - "62.5" or "123.0" for CTCSS tones
// - "D023N" or "D754I" for DCS tones
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

fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    dprintln!(opt.verbose, 4, "{:?}", record);

    let mut channel = Channel::default();

    // check if the record is a VFO
    if record.get("No.").unwrap().starts_with("VFO") {
        return Ok(channel);
    }
    // HD1 CPS is garbage and exports all rows regardless of content
    if record.get("No.").unwrap().is_empty() {
        return Ok(channel);
    }

    // shared fields
    channel.index = record.get("No.").unwrap().parse::<u32>()?;
    channel.name = record.get("Channel Alias").unwrap().to_string();
    channel.mode = match record.get("Channel Type").unwrap().as_str() {
        "Analog CH" => ChannelMode::FM,
        "Digital CH" => ChannelMode::DMR,
        _ => return Err(format!("Unrecognized channel type: {}", record.get("Channel Type").unwrap()).into()),
    };
    channel.frequency_rx = Decimal::from_str(record.get("Rx Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.frequency_tx = Decimal::from_str(record.get("Tx Frequency").unwrap())? * Decimal::new(1_000_000, 0);
    channel.rx_only = record.get("Tx Authority").unwrap() == "Prohibit TX";
    if channel.mode == ChannelMode::FM { // FM specific fields
        channel.fm = Some(FmChannel {
            // strip the 'K' from the end of the value
            bandwidth: Decimal::from_str(record.get("Band Width").unwrap().strip_suffix("K").unwrap())?,
            squelch: Squelch {
                default: true,
                percent: None,
            },
            tone_rx: parse_tone(record.get("Dec QT/DQT").unwrap()),
            tone_tx: parse_tone(record.get("Enc QT/DQT").unwrap()),
        });
    } else if channel.mode == ChannelMode::DMR { // DMR specific fields
        channel.dmr = Some(DmrChannel {
            color_code: record.get("Color Code").unwrap().parse::<u8>()?,
            timeslot: record.get("Slot").unwrap().strip_prefix("Slot").unwrap().parse::<u8>()?,
            talkgroup: None,
            talkgroup_list: None,
        });
    }

    Ok(channel)
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
        source: format!("{}", Path::new(file!()).file_stem().unwrap().to_str().unwrap()),
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

    // check for Channels.csv
    let channels_path: PathBuf = input_path.join("Channels.csv");
    if !channels_path.exists() {
        return Err("Channels.csv not found".into());
    } else {
        dprintln!(opt.verbose, 3, "Reading {}", channels_path.display());
        let mut reader = csv::Reader::from_path(&channels_path)?;
        for result in reader.deserialize() {
            let record: CsvRecord = result?;
            // convert from CSV record to Channel struct
            let channel = parse_channel_record(&record, &opt)?;
            if channel.index > 0 {
                // append to codpelug.channels
                codeplug.channels.push(channel);
            }
        }
    }

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////

pub fn write_channels(codeplug: &Codeplug, path: &PathBuf, opt: &Opt) -> Result<(), Box<dyn Error>> {
    dprintln!(opt.verbose, 3, "{}:{}()", file!(), function!());
    dprintln!(opt.verbose, 1, "Writing {}", path.display());

    let mut writer = csv::WriterBuilder::new()
    .from_path(path)?;

    // write header
    writer.write_record(&[
        "No.",
        "Channel Type",
        "Channel Alias",
        "Rx Frequency",
        "Tx Frequency",
        "Tx Power",
        "TOT",
        "VOX",
        "VOX Level",
        "Scan Add/Step",
        "Channel Work Alone",
        "Default to Talkaround",
        "Band Width",
        "Dec QT/DQT",
        "Enc QT/DQT",
        "Tx Authority",
        "Relay",
        "Work Mode",
        "Slot",
        "ID Setting",
        "Color Code",
        "Encryption",
        "Encryption Type",
        "Encryption Key",
        "Promiscuous",
        "Tx Authority",
        "Kill Code",
        "WakeUp Code",
        "Contacts",
        "Rx Group Lists",
        "Group Lists 1",
        "Group Lists 33",
        "GPS",
        "Send GPS Info",
        "Receive GPS Info",
        "GPS Timing Report",
        "GPS Timing Report TX Contact",
    ])?;

    for channel in &codeplug.channels {
        dprintln!(opt.verbose, 4, "Writing channel {:width$}: {}", channel.index, channel.name, width = get_props().channel_index_width);
        dprintln!(opt.verbose, 4, "    {:?}", channel);

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

    // write to Channels.csv
    let mut channels_path: PathBuf = opt.output.clone().unwrap();
    channels_path.push(if opt.excel { "Channels2.CSV" } else { "Channels.CSV" });
    write_channels(codeplug, &channels_path, opt)?;

    Ok(())
}
