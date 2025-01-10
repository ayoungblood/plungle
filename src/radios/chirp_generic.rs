// src/radios/opengd77_rt3s.rs
// reference https://burntsushi.net/csv/ for CSV parsing technique

use std::error::Error;
// use std::fs;
// use std::path::PathBuf;
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
        props.channels_max = 1024;
        props.channel_name_width_max = 16;
        // dynamically set
        props.channel_index_width = (props.channels_max as f64).log10().ceil() as usize;
        props
    })
}

// CSV Export Format
// CHIRP next-20241108
// CHIRP exports a single CSV file:
// - Location: channel index
// - Name: channel name
// - Frequency: receive frequency in MHz
// - Duplex: [+, -, (blank), split, off]
// - Offset: transmit offset in MHz, typ [0, 0.6, 5]
// - Tone: complicated
//     Tone -
// - rToneFreq: RX CTCSS frequency in Hz, 88.5 default
// - cToneFreq: TX(?) CTCSS frequency in Hz, 88.5 default
// - DtcsCode: DCS code, 23 default
// - DtcsPolarity: DCS polarity, NN default
// - RxDtcsCode: RX DCS code, 23 default
// - CrossMode: [Tone-Tone, ??]
// - Mode: [FM, NFM, ??]
// - TStep: default 5
// - Skip: [(blank), ??]
// - Power: power in watts with W suffix, e.g. [1.0W, 4.0W, 50W]
// - Comment: blank by default
// - URCALL: blank by default
// - RPT1CALL: blank by default
// - RPT2CALL: blank by default
// - DVCODE: blank by default

type CsvRecord = HashMap<String, String>;

// READ ///////////////////////////////////////////////////////////////////////

pub fn parse_channel_record(record: &CsvRecord, opt: &Opt) -> Result<Channel, Box<dyn Error>> {
    dprintln!(opt.verbose, 4, "{:?}", record);

    let mut channel = Channel::default();

    // chirp uses zero-index, +1 to match other CPS
    if record.get("Mode").unwrap().as_str() == "NFM" || record.get("Mode").unwrap().as_str() == "FM" {
        channel.index = record.get("Location").unwrap().parse::<u32>()? + 1;
        channel.name = record.get("Name").unwrap().to_string();
        channel.mode = ChannelMode::FM;
        channel.frequency_rx = Decimal::from_str(record.get("Frequency").unwrap())? * Decimal::new(1_000_000, 0);
        dprintln!(opt.verbose, 4, "RX: {}", record.get("Frequency").unwrap());
        let offset = match record.get("Offset").unwrap().as_str() {
            "" => Decimal::new(0, 0),
            s => Decimal::from_str(s)? * Decimal::new(1_000_000, 0),
        };
        channel.frequency_tx = match record.get("Duplex").unwrap().as_str() {
            "+" => channel.frequency_rx + offset,
            "-" => channel.frequency_rx - offset,
            "split" => offset,
            "off" => channel.frequency_rx,
            _ => channel.frequency_rx,
        };
        channel.rx_only = match record.get("Duplex").unwrap().as_str() {
            "off" => true,
            _ => false,
        };
    } else {
        cprintln!(ANSI_C_RED, "Unsupported mode: {}", record.get("Mode").unwrap());
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

    // check that the input path is a file
    let input_path = match &opt.input {
        Some(path) => {
            if path.is_file() {
                path
            } else {
                cprintln!(ANSI_C_RED, "You lied to me when you told me this was a file: {}", path.display());
                return Err("Bad input path".into());
            }
        }
        None => return Err("Bad input path".into()),
    };

    dprintln!(opt.verbose, 3, "Reading from: {}", input_path.display());
    let mut reader = csv::Reader::from_path(input_path)?;
    for result in reader.deserialize() {
        let record: CsvRecord = result?;
        // convert from CSV record to Channel
        let channel = parse_channel_record(&record, &opt)?;
        if channel.index > 0 {
            // append to codeplug.channels
            codeplug.channels.push(channel);
        }
    }

    Ok(codeplug)
}

// WRITE //////////////////////////////////////////////////////////////////////